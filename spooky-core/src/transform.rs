// When std is available, we simply alias to the Bevy Transform.
#[cfg(feature = "std")]
pub type UnifiedTransform = bevy_transform::components::Transform;

// When in no_std mode, alias to your own type.
#[cfg(not(feature = "std"))]
pub type UnifiedTransform = crate::systems::setup::NoStdTransform;
