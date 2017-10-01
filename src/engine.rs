//! Engine context passed into the active game state.

use std::sync::Arc;
use std::time::Duration;

use rayon::ThreadPool;
use renderer::types::Window;
use shrev::EventHandler;

use ecs::World;
use ecs::rendering::WindowModifierEvent;

/// User-facing engine handle.
pub struct Engine {
    /// Current delta time value.
    pub delta: Duration,
    /// Thread pool.
    pub pool: Arc<ThreadPool>,
    /// World.
    pub world: World,
}

impl Engine {
    /// Creates a new engine context.
    pub(crate) fn new(pool: Arc<ThreadPool>, world: World) -> Self {
        Engine {
            delta: Duration::from_secs(0),
            pool: pool,
            world: world,
        }
    }

    /// Sends a command to the game's inner window
    pub fn window_command<F>(&mut self, command: F)
    where
        F: Fn(&mut Window) -> () + Send + Sync + 'static,
    {
        self.world
            .res
            .try_fetch_mut::<EventHandler<WindowModifierEvent>>(0)
            .expect("Command queue for the Window not found!")
            .write_single(WindowModifierEvent {
                modify: Box::new(command),
            });
    }
}
