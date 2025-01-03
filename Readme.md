# Examples re-implemented using the ECS paradigm

As in [actor_model_demo](https://github.com/RobertJacobsonCDC/actor_model_demo),
I re-implement the `basic-infection` example (and possibly others) using
the industrial strength _Entity-Component-System_ (ECS) library [Bevy
ECS](https://docs.rs/bevy_ecs/latest/bevy_ecs/index.html). Bevy ECS and the ECS concept
more generally come out of the game development world in which very sophisticated
data access patterns are required under very strict performance constraints.

# Two and a Half Interacting Requirements

The point is to illustrate the second of the two (and a bit) major requirements of a discrete
event agent modeling framework, namely the data model. As before, here are a few disjointed
thoughts on these requirements, a kind of brain dump written mostly for my own benefit.

# Recall key concepts

Entity-Component-System consists of three primary interacting concepts:

- **Entity:** An entity represents a general-purpose object. In a game engine context, for example, every coarse game object is represented as an entity. Think of these as the rows of the database. A person will generally be an entity.

- **Component:** A component characterizes an entity as possessing a particular aspect, and holds the data needed to model that aspect. For example, every game object that can take damage might have a Health component associated with its entity. Think of these as the columns of a database. A person entity, for example, might have a component for age, weight, vaccination status, and so forth.

- **System:** A system is a process which acts on all entities with the desired components. For example, a physics system may query for entities having mass, velocity and position components, and iterate over the results doing physics calculations on the set of components for each entity. Generally speaking, systems are just regular functions. Continuing the database analogy, think of systems as SQL queries. Like a SQL query, a system might mutate data, or it might not.

# Relationship between entities, components, and Rust datatypes

In a normal Rust program, a person might be represented with a struct which stores values of different properties of the person:

```rust
struct Person {
  sex             : Sex,
  ethnicity       : Ethnicity,
  age             : u8,
  weight          : u16,
  infection_status: InfectionStatus,
  // possibly other fields...
}

// Later we might create an instance of a `Person`...
let person = Person {
  sex             : Sex::Male,
  ethnicity       : Ethnicity::Caucasion,
  age             : 43,
  weight          : 215,
  infection_status: InfectionStatus::Susceptible,
};
```

With an ECS, an entity representing a person is just a group of *components*. Components are exactly those pieces of data associated with a person that need to be processed or queried by ECS systems (functions). An instance of an entity is "spawned" in the "world":

```rust
// There is generally only one `World` instance for a single simulation to which mutable access is given as necessary. 
let mut world = World::default();
let person    = Person::new(); 
// …or perhaps a person instance has been passed to this code via a method call.

world.spawn()
		 .insert(person)
     .insert(InfectionStatus::Susceptible)
     .insert(Weight(70.0))
     .insert(Age(30));
```

Don't be fooled by the word *insert*: Only a single entity is being created here. What is being inserted are the components of the entity created by `world.spawn()`, not additional entities. Indeed, even the `Person` instance `person` is being inserted as a component.

In fact, if all of the data fields of the `Person` struct are really components, and if the only entities that exist are people, there really is no need for the `Person` struct at all! A person is completely represented by the entity alone.

### When to put data into the fields of a struct vs. as components of an entity

In Bevy ECS, choosing between storing data in a struct (e.g., `Person`) versus using components attached to an entity depends on the nature of the data and how you plan to use it within the simulation.

#### Use a Struct (e.g., `Person`) for Entity Metadata or Non-ECS Data

You can use a struct like `Person` to group metadata or non-ECS-related data that you want to associate with the entity. This is a good choice when:

- **The data represents a complex entity description:** If the data involves complex relationships or multiple fields that don't change often and don't need to be processed independently, it makes sense to store it in a struct.
- **You need to encapsulate data that doesn't fit naturally into ECS components:** For example, data like `Sex`, `Ethnicity`, or other personal characteristics that are set once and don't require frequent updates or ECS-based processing. These fields might not be used by systems that run on the ECS, but could be useful for querying, displaying, or interacting with the entity in other ways.
- **You don't need to query or operate on the data directly in the ECS systems:** If the data doesn't need to be manipulated or queried during the ECS-based processing (e.g., systems that operate on components), a struct is a good choice.
- **You want to hold data related to a specific instance:** The struct can act as a way to logically group data that's specific to an individual, but without needing the ECS framework to track its changes or updates.

For example:

```rust
struct Person {
    sex: Sex,
    ethnicity: Ethnicity,
}

impl Person {
    fn new(sex: Sex, ethnicity: Ethnicity) -> Self {
        Person { sex, ethnicity }
    }
}

// When spawning an entity:
let person = Person::new(Sex::Male, Ethnicity::Caucasian);
world.spawn()
    .insert(person)  // Associate the Person struct with the entity
    .insert(InfectionStatus::Susceptible)
    .insert(Weight(70.0))
    .insert(Age(30));
```



#### Use Components for Entity Data That Needs to Be Processed or Queried

Components are more appropriate for data that needs to be processed, updated, or queried by Bevy ECS systems. This is the heart of ECS architecture: you store data in components and process it in systems. Use components when:

- **The data is part of the ECS processing logic:** If the data will be frequently updated by systems or interacted with in a way that ECS needs to track, it should be a component. For example, `InfectionStatus`, `Weight`, and `Age` are perfect as components because systems may frequently update them (e.g., when an entity becomes infected or ages).
- **You need to query the data efficiently in ECS systems:** Bevy ECS is optimized for querying components in bulk. If you need to perform operations on entities based on their components (e.g., finding all `Infected` people, or all `Weight` values above a certain threshold), components are the right choice.
- **The data needs to be updated or modified frequently during simulation:** Data like health status (`InfectionStatus`), physical attributes (`Weight`, `Age`), or simulation-related properties should be components, as they will be updated by systems and likely need to be queried.
- **You want to benefit from ECS's performance:** Bevy ECS is optimized to handle large numbers of entities with many components efficiently. By using components for data that needs to be manipulated or queried, you benefit from this optimization.

For example:

```rust
#[derive(Component)]
struct InfectionStatus {
    status: Status,  // Could be Susceptible, Infected, Recovered
}

#[derive(Component)]
struct Weight(f64);

#[derive(Component)]
struct Age(u32);

// When spawning an entity:
world.spawn()
    .insert(InfectionStatus { status: Status::Susceptible })
    .insert(Weight(70.0))
    .insert(Age(30));
```



### When to still use a struct even if it has no data fields

A field-less struct can act as a label for a category of entity. For example, it might make sense to have a `Person` entity and an `Animal` entity to represent livestock or family pets. Defining a field-less struct for each of these entity categories allows you to query for just people or just animals.

A struct with no fields is a *zero-cost abstraction* in Rust. This is because, in the context of Rust’s memory layout, a struct with no fields has no data associated with it, and therefore, it doesn’t occupy any memory or incur any runtime overhead. It’s just a *marker type* or a *tag* that can be used to categorize entities, but it doesn’t introduce any additional complexity or cost.

Another benefit of using a fieldless struct in an entity is to organize methods (functions) that act on the particular category of entity that carries the struct as a marker type. For example, entities with the `Person` type might be serialized in a different way from entities with the `Animal` type.

# Rosetta Stone

This "Rosetta Stone" is my attempt at a translation dictionary, but the concepts are sliced up so differently that this might be more confusing than helpful. 

| Concept              | Ixa      | Bevy ECS         | Description                                             |
|:---------------------|:---------|:-----------------|:--------------------------------------------------------|
| a row index          | `Person` | `Entity`         | a tuple of properties represented by a handle           | 
| mutator of the world | `Plan`   | `System`         | a function that processes entities/components           |
| timeline             | `Queue`  | `Schedule`       | priority queue of callbacks, min-prioritized by time    |
| event                | `Plan`   | `System`         | a mutator with an associated time                       |
| query                | `Query`  | `Query`/`System` | function acting on selection of rows (possibly mutably) |
| When a function fires | `ExecutionPhase` | `Schedule`        | before time, at time, after time                        |
<hr>

Begin boilerplate…

**General disclaimer** This repository was created for use by CDC programs to collaborate on public health related projects in support of the [CDC mission](https://www.cdc.gov/about/organization/mission.htm).  GitHub is not hosted by the CDC, but is a third party website used by CDC and its partners to share information and collaborate on software. CDC use of GitHub does not imply an endorsement of any one particular service, product, or enterprise.

## Public Domain Standard Notice
This repository constitutes a work of the United States Government and is not
subject to domestic copyright protection under 17 USC § 105. This repository is in
the public domain within the United States, and copyright and related rights in
the work worldwide are waived through the [CC0 1.0 Universal public domain dedication](https://creativecommons.org/publicdomain/zero/1.0/).
All contributions to this repository will be released under the CC0 dedication. By
submitting a pull request you are agreeing to comply with this waiver of
copyright interest.

## License Standard Notice
The repository utilizes code licensed under the terms of the Apache Software
License and therefore is licensed under ASL v2 or later.

This source code in this repository is free: you can redistribute it and/or modify it under
the terms of the Apache Software License version 2, or (at your option) any
later version.

This source code in this repository is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the Apache Software License for more details.

You should have received a copy of the Apache Software License along with this
program. If not, see http://www.apache.org/licenses/LICENSE-2.0.html

The source code forked from other open source projects will inherit its license.

## Privacy Standard Notice
This repository contains only non-sensitive, publicly available data and
information. All material and community participation is covered by the
[Disclaimer](DISCLAIMER.md)
and [Code of Conduct](code-of-conduct.md).
For more information about CDC's privacy policy, please visit [http://www.cdc.gov/other/privacy.html](https://www.cdc.gov/other/privacy.html).

## Contributing Standard Notice
Anyone is encouraged to contribute to the repository by [forking](https://help.github.com/articles/fork-a-repo)
and submitting a pull request. (If you are new to GitHub, you might start with a
[basic tutorial](https://help.github.com/articles/set-up-git).) By contributing
to this project, you grant a world-wide, royalty-free, perpetual, irrevocable,
non-exclusive, transferable license to all users under the terms of the
[Apache Software License v2](http://www.apache.org/licenses/LICENSE-2.0.html) or
later.

All comments, messages, pull requests, and other submissions received through
CDC including this GitHub page may be subject to applicable federal law, including but not limited to the Federal Records Act, and may be archived. Learn more at [http://www.cdc.gov/other/privacy.html](http://www.cdc.gov/other/privacy.html).

## Records Management Standard Notice
This repository is not a source of government records, but is a copy to increase
collaboration and collaborative potential. All government records will be
published through the [CDC web site](http://www.cdc.gov).

## Additional Standard Notices
Please refer to [CDC's Template Repository](https://github.com/CDCgov/template) for more information about [contributing to this repository](https://github.com/CDCgov/template/blob/main/CONTRIBUTING.md), [code of conduct](https://github.com/CDCgov/template/blob/main/code-of-conduct.md), and other related documentation.
