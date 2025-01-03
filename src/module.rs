/*!

A `Module` is a thing that can be initialized with a `World`, possibly mutating the `World`, optionally adding
systems to the `Schedule` to be run on the `World` in the event loop. A `Module` is the primary building block from
which a model is
constructed.

The only required function is the constructor, which has a special form that the `Model` uses to initialize
the `Module` within itself. An alternative design could be that the `Module`'s constructor takes a `&mut Model`,
but that spreads the implementation details, at least the public API, of `Model` everywhere, whereas
`World` has (for our purposes) a fixed API.

*/

use bevy_ecs::prelude::*;

pub trait Module {
  /// This method is a constructor and is called to initialize a new `Module` with the provided `World`.
  fn initialize_with_world(world: &mut World, schedule: &mut Schedule);
}
