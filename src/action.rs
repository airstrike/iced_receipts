//! A common API for returning [`Task`](iced::Task)s and/or generic
//! `Instruction`s from views.
//!
//! The `Action` type provides a unified way to tell an ancestor component how
//! to modify its state per some `Instruction` and/or provide a [`Task`] which
//! should be returned to the [`iced`] runtime so it can be executed.
//!
//! Examples of instructions include navigating to a different screen, saving
//! changes, or starting an edit. These instructions work directly in the
//! context of some ancestor component, so they do not need to be sent to the
//! runtime.  An `Instruction` is just a way to convey an action to an ancestor.
//! Enumerating these instructions in a single (generic) type allows for a more
//! consistent API.
//!
//! A [`Task`] is a way to perform some asynchronous instruction, such as
//! fetching data from a server or something as simple as changing the currently
//! focused widget. These must be returned to the runtime in order for them to
//! be executed.
//!
//! In many instances, you will want to return both an `Instruction` and a
//! [`Task`].  Say, for example, when you'd like to navigate to a different
//! screen *and* focus the first input field. This can be achieved with e.g.
//! `Action::instruction(Instruction::NavigateToScreen).with_task(focus_first_input())`.
//!
//! Generally speaking, an `Action` will alwys be created as a result of
//! processing some other `Message`. For example, a
//! `button("Back").on_press(Message::Back)` will emit a `Message::Back` when
//! pressed, which will then be processed by a `fn update(message: Message) ->
//! Action` function, returning e.g.  `Action::instruction(Instruction::Back)`.
//!
//! It is the responsibility of the ancestor component to handle the
//! `Instruction` that is returned from that child view. To make the code easier
//! to follow, it may be advantageous to define a separate `fn
//! perform(instruction: Instruction) -> Task` function to handle any
//! instructions returned by the child view. In some cases, those instructions
//! may result in yet another [`Task`], which would require the parent component
//! to chain the tasks together. An example of this can be be seen in the `fn
//! update` function in `src/main.rs`.
//!
//! This design pattern is common in many [`iced`] applications, although the
//! exact implementation may vary. It is often the case that the `Action` is
//! simply an enum which contains either instructions or tasks, but our proposed
//! design allows for more flexibility and clarity at the expense of slightly
//! more boilerplate upfront, which we think is a worthy tradeoff *for apps
//! which are expected to grow in complexity*. For smaller apps, the simpler
//! enum-based approach may be more appropriate, for example:
//!
//! ```rust
//! pub enum Action {
//!     Instruction1,
//!     Instruction2,
//!     Run(Task<Message>),
//! }
//! ```
use iced::Task;
use std::fmt;

pub struct Action<I, Message> {
    pub instruction: Option<I>,
    pub task: Task<Message>,
}

impl<I, Message> Action<I, Message> {
    /// Create a new `Action` with no `Instruction` or [`Task`](iced::Task).
    pub fn none() -> Self {
        Self {
            instruction: None,
            task: Task::none(),
        }
    }

    /// Create a new `Action` with an `Instruction` and a [`Task`](iced::Task).
    pub fn new(instruction: I, task: Task<Message>) -> Self {
        Self {
            instruction: Some(instruction),
            task,
        }
    }

    /// Create a new `Action` with an `Instruction` to be handled by some ancestor
    /// component.
    pub fn instruction(instruction: I) -> Self {
        Self {
            instruction: Some(instruction),
            task: Task::none(),
        }
    }

    /// Create a new `Action` with a [`Task`](iced::Task).
    pub fn task(task: Task<Message>) -> Self {
        Self {
            instruction: None,
            task,
        }
    }

    /// Map the message of the `Action`'s [`Task`](iced::Task) to a different type.
    pub fn map<N>(
        self,
        f: impl Fn(Message) -> N + Send + 'static,
    ) -> Action<I, N>
    where
        Message: Send + 'static,
        N: Send + 'static,
    {
        Action {
            instruction: self.instruction,
            task: self.task.map(f),
        }
    }

    /// Maps the `Instruction` of the `Action` to a different type.
    pub fn map_instruction<N>(
        self,
        f: impl Fn(I) -> N + Send + 'static,
    ) -> Action<N, Message>
    where
        I: Send + 'static,
        N: Send + 'static,
    {
        Action {
            instruction: self.instruction.map(f),
            task: self.task,
        }
    }

    /// Sets the `Instruction` of an `Action`.
    pub fn with_instruction(mut self, instruction: I) -> Self {
        self.instruction = Some(instruction);
        self
    }

    /// Sets the [`Task`](iced::Task) of an `Action`.
    pub fn with_task(mut self, task: Task<Message>) -> Self {
        self.task = task;
        self
    }
}

impl<Instruction: fmt::Debug, Message> fmt::Debug
    for Action<Instruction, Message>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("instruction", &self.instruction)
            .finish()
    }
}
