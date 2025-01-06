/*!

A `Module` is a thing that can be initialized with a `World`, possibly mutating the `World`, optionally adding
systems to the `Schedule` to be run on the `World` in the event loop. A `Module` is the primary building block from
which a model is constructed.

The only required function is the initializer. It's not clear if this is a good way to initialize a module
(into a world). We want to be flexible and general enough to enable many different kinds of modules, but
we also do not necessarily want to give the module too much access. Maybe the latter isn't a concern, since
they have exclusive world access anyway--but they don't have access to the event loop and schedule. Another
downside to this choice is the assumption that the module insert itself into the world so that the world owns it.

An alternative design could be that the `Module`'s constructor takes a `&mut Model`, but that spreads the
implementation details, at least the public API, of `Model` everywhere, whereas `World` has (for our purposes)
a fixed API. Full Bevy takes this route: https://bevy-cheatbook.github.io/programming/plugins.html.

*/

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::SystemConfigs;

pub trait Module {
  /// This method is a constructor and is called to initialize a new `Module` with the provided `World`.
  fn initialize_with_world(self, world: &mut World) -> Option<SystemConfigs>;
}
