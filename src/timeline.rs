/*!

A `Timeline` is a min priority queue of systems (in the ECS sense of the word
system). When a new timeline event is ready to be triggered, the system (the
function) associated to the smallest time will be popped off the timeline and
executed. The timeline keeps track of the current time when it pops an event.

Note that Bevy ECS uses the word _event_ to refer to a message that can be
passed between systems, which is completely different from our use here.

An `Event` is just a struct to hold a `(Time, System)` pair.

*/

use std::collections::BinaryHeap;

use ordered_float::OrderedFloat;
use bevy_ecs::{
  prelude::*,
  // system::ExclusiveSystemParamFunction
};
use bevy_ecs::schedule::SystemConfigs;
use crate::{
  model::{ExecutionPhase, ModelControl},
  module::Module,
  timeline_event::Event
};

/// `Time` is just an alias for a hashable totally ordered float.
pub type Time = OrderedFloat<f64>;

/// `Timeline` is a thin wrapper around `BinaryHeap<Event>` that keeps track of the "current time" as events are popped.
#[derive(Resource)]
pub struct Timeline {
  now        : Time,
  event_queue: BinaryHeap<Event>,
}

impl Default for Timeline {
  fn default() -> Self {
    Self {
      now        : Time::default(),
      event_queue: BinaryHeap::new(),
    }
  }
}


impl Timeline {

  #[must_use]
  #[inline(always)]
  pub fn now(&self) -> Time {
    self.now
  }

  // We might not want to allow this.
  #[allow(unused)]
  #[inline(always)]
  pub fn set_now(&mut self, new_time: Time) -> Time {
    self.now = new_time;
    new_time
  }

  #[inline(always)]
  pub fn push(&mut self, event: Event) {
    self.event_queue.push(event)
  }

  /// Pop's the next event, updating `self.now` to the new time associated to the event.
  #[inline(always)]
  pub fn pop(&mut self) -> Option<Event> {
    let popped = self.event_queue.pop();
    if let Some(Event { time, .. }) = &popped {
      self.now = time.clone();
    }

    popped
  }
}

impl Module for Timeline {
  fn initialize_with_world(self, world: &mut World) -> Option<SystemConfigs> {
    #[cfg(feature = "print_messages")]
    println!("Initialized module Timeline");

    // Insert the Timeline resource into the World
    world.insert_resource(Timeline::default());

    // There is only one system in our implementation, namely the one that runs (at most) a single event.
    Some(run_timeline_event.in_set(ExecutionPhase::Normal))
  }
}

/// The `System` for the `Timeline` module. It runs a scheduled event, if one exists.
fn run_timeline_event(
  mut timeline: ResMut<Timeline>,
  mut model_control: ResMut<ModelControl>,
  mut commands: Commands,
) {
  if let Some(Event{command, ..}) = timeline.pop() {
    commands.queue(command);
  }
  else {
    // In this model this only happens if there is a bug, which nobody on our time would ever write.
    #[cfg(feature = "print_messages")]
    println!("Timeline empty. Requesting Abort.");
    *model_control = ModelControl::Aborted;
  }
}


#[cfg(test)]
mod tests {
  use bevy_ecs::prelude::World;
  use super::*;

  #[test]
  fn test_get_set_time() {
    let mut world = World::default();

    // Insert Time resource into the world
    world.insert_resource(Timeline::default());

    {
      let mut timeline = world.get_resource_mut::<Timeline>().unwrap();
      assert_eq!(timeline.now(), OrderedFloat(0.0));

      // Modify the time.
      timeline.set_now(OrderedFloat(2.0 * std::f64::consts::PI));
    }

    // Get the current time
    let time = world.get_resource::<Timeline>().unwrap();
    assert_eq!(time.now(), OrderedFloat(2.0 * std::f64::consts::PI));
  }
}
