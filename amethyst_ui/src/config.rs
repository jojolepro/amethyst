//! Module containing the Amethyst ui configurations.

#[derivative(Default)]
#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct UiConfig {
	#[derivative(Default = "0.5")]
	pub cursor_blink_rate: f32,
	pub cursor_color: Rgba,
}