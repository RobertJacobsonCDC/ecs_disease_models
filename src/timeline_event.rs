use std::cmp::{Ordering, Reverse};

use bevy_ecs::prelude::*;

use crate::timeline::Time;

pub struct Event {
  pub time  : Time,
  // ToDo: This might not be the right type, here. We want a thing that is
  //       Send and Sync with which we can put a command on the command
  //       queue.
  pub command: Box<dyn FnOnce(&mut World) + Send + Sync>,
  // We could also record the actor who scheduled the event, etc.
}

// impl Command for Event {
//   fn apply(self, world: &mut World) {
//     #[cfg(feature = "print_messages")]
//     println!("Executing Event at {:?}", self.time);
//     (self.command)(world);
//   }
// }

// Implements ordering of events in the timeline's priority queue. This is necessary because
// `BinaryHeap` is a max heap, not a min heap, and we want a min heap.
//
// Be warned that `Event`s are equal if they are scheduled at the same time regardless of payload.
impl PartialEq for Event {
  fn eq(&self, other: &Self) -> bool {
    self.time == other.time
  }
}

impl Eq for Event{}

impl PartialOrd for Event {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for Event {
  fn cmp(&self, other: &Self) -> Ordering {
    Reverse(self.time).cmp(&Reverse(other.time))
  }
}
