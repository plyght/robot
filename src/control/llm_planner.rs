use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use std::env;

use anyhow::Result;
use crate::vision::DetectedObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandPose {
    pub palm_center: (f32, f32, f32),
    pub wrist_position: (f32, f32, f32),
    pub finger_tips: Vec<(f32, f32, f32)>,
    pub is_open: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneState {
    pub target_object: DetectedObject,
    pub object_depth_cm: f32,
    pub hand_pose: Option<HandPose>,
    pub other_objects: Vec<DetectedObject>,
    pub camera_fov_horizontal: f32,
    pub camera_fov_vertical: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementCommand {
    pub action: MovementAction,
    pub parameters: MovementParameters,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MovementAction {
    MoveToPosition,
    OpenHand,
    CloseHand,
    Grasp,
    Release,
    RotateWrist,
    Approach,
    Retreat,
    Wait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementParameters {
    pub target_x_cm: Option<f32>,
    pub target_y_cm: Option<f32>,
    pub target_z_cm: Option<f32>,
    pub wrist_pitch: Option<f32>,
    pub wrist_roll: Option<f32>,
    pub grip_strength: Option<f32>,
    pub duration_ms: Option<u64>,
}

pub struct LlmPlanner {
    client: Client<OpenAIConfig>,
    model: String,
}

impl LlmPlanner {
    pub fn new() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable not set"))?;

        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);

        Ok(Self {
            client,
            model: "gpt-5-nano-2025-08-07".to_string(),
        })
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub async fn generate_movement_plan(
        &self,
        scene: &SceneState,
    ) -> Result<Vec<MovementCommand>> {
        let prompt = self.build_prompt(scene);

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(SYSTEM_PROMPT)
                        .build()?,
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(prompt)
                        .build()?,
                ),
            ])
            .temperature(0.3)
            .max_tokens(1000u32)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| anyhow::anyhow!("No response from LLM"))?;

        self.parse_commands(content)
    }

    fn build_prompt(&self, scene: &SceneState) -> String {
        let hand_info = match &scene.hand_pose {
            Some(pose) => format!(
                "Hand detected at position: ({:.1}, {:.1}, {:.1}) cm, Palm center: ({:.1}, {:.1}, {:.1}), Open: {}, Confidence: {:.2}",
                pose.wrist_position.0,
                pose.wrist_position.1,
                pose.wrist_position.2,
                pose.palm_center.0,
                pose.palm_center.1,
                pose.palm_center.2,
                pose.is_open,
                pose.confidence
            ),
            None => "Hand position unknown - will need to estimate starting position".to_string(),
        };

        let other_objects_info = if scene.other_objects.is_empty() {
            "No other objects detected in scene".to_string()
        } else {
            format!(
                "Other objects in scene: {}",
                scene
                    .other_objects
                    .iter()
                    .map(|obj| format!(
                        "{} at ({}, {}) with depth ~{}cm",
                        obj.label,
                        obj.bounding_box.x,
                        obj.bounding_box.y,
                        obj.distance
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        format!(
            r#"SCENE ANALYSIS:

TARGET OBJECT:
- Type: {}
- Confidence: {:.2}
- Position in frame: ({}, {})
- Bounding box size: {}x{} pixels
- Estimated depth: {:.1} cm

HAND STATE:
{}

ENVIRONMENT:
{}
- Camera FOV: {:.1}° horizontal, {:.1}° vertical

TASK: Generate a sequence of movement commands to safely grasp the target object.

Respond ONLY with valid JSON in this exact format:
{{
  "commands": [
    {{
      "action": "MoveToPosition" | "OpenHand" | "CloseHand" | "Grasp" | "Release" | "RotateWrist" | "Approach" | "Retreat" | "Wait",
      "parameters": {{
        "target_x_cm": float | null,
        "target_y_cm": float | null,
        "target_z_cm": float | null,
        "wrist_pitch": float | null,
        "wrist_roll": float | null,
        "grip_strength": float | null,
        "duration_ms": int | null
      }},
      "reasoning": "brief explanation"
    }}
  ]
}}

Consider:
1. Hand must approach from above or side depending on object type
2. Grasp force appropriate for object (fragile vs sturdy)
3. Avoid collisions with other objects
4. Smooth motion trajectory
5. If hand position unknown, start with safe default approach"#,
            scene.target_object.label,
            scene.target_object.confidence,
            scene.target_object.bounding_box.x,
            scene.target_object.bounding_box.y,
            scene.target_object.bounding_box.width,
            scene.target_object.bounding_box.height,
            scene.object_depth_cm,
            hand_info,
            other_objects_info,
            scene.camera_fov_horizontal,
            scene.camera_fov_vertical,
        )
    }

    fn parse_commands(&self, content: &str) -> Result<Vec<MovementCommand>> {
        let trimmed = content.trim();
        let json_start = trimmed
            .find('{')
            .ok_or_else(|| anyhow::anyhow!("No JSON found in response"))?;
        let json_end = trimmed
            .rfind('}')
            .ok_or_else(|| anyhow::anyhow!("No JSON found in response"))?;

        let json_str = &trimmed[json_start..=json_end];

        #[derive(Deserialize)]
        struct Response {
            commands: Vec<MovementCommand>,
        }

        let response: Response = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response: {}", e))?;

        Ok(response.commands)
    }
}

const SYSTEM_PROMPT: &str = r#"You are a robot movement planner for a 5-finger robotic hand.

CAPABILITIES:
- 5 articulated fingers (Thumb, Index, Middle, Ring, Pinky)
- Each finger has 3 joints with 0-90° range
- 2-axis wrist (pitch and roll)
- Servo-based position control (precise but not force-sensing)

COORDINATE SYSTEM:
- X: left (-) to right (+)
- Y: down (-) to up (+)
- Z: camera (0) to away (+)
- All measurements in centimeters

SAFETY CONSTRAINTS:
- Maximum reach: ~30cm from wrist
- Approach speed: slow for fragile objects, normal for sturdy
- Never command movements that would collide with other objects
- If uncertain, use conservative grip strength

OUTPUT FORMAT:
Return ONLY valid JSON with movement commands. Each command must have:
- action: one of the predefined action types
- parameters: relevant numerical values (use null for unused parameters)
- reasoning: 1-2 sentence explanation

Be concise and direct. Prioritize safety and success rate over speed."#;
