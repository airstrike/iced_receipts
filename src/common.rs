//! A common API for returning [`Task`](iced::Task)s and/or generic `Operation`s
//! from views.
//!
//! The `Action` type provides a unified way to tell an ancestor component
//! how to modify its state per some `Operation` and/or provide a [`Task`]
//! which should be returned to the [`iced`] runtime so it can be executed.
//!
//! Examples of operations include navigating to a different screen, saving
//! changes, or starting an edit. These operations work directly in the context
//! of some ancestor component, so they do not need to be sent to the runtime.
//! An `Operation` is just a way to convey an action to an ancestor. Enumerating
//! these operations in a single (generic) type allows for a more consistent API.
//!
//! A [`Task`] is a way to perform some asynchronous operation, such as fetching
//! data from a server or something as simple as changing the currently focused
//! widget. These must be returned to the runtime in order for them to be
//! executed.
//!
//! In many instances, you will want to return both an `Operation` and a [`Task`].
//! Say, for example, when you'd like to navigate to a different screen *and*
//! focus the first input field. This can be achieved with e.g.
//! `Action::operation(Operation::NavigateToScreen).with_task(focus_first_input())`.
//!
//! Generally speaking, an `Action` will alwys be created as a result of processing
//! some other `Message`. For example, a `button("Back").on_press(Message::Back)`
//! will emit a `Message::Back` when pressed, which will then be processed by
//! a `fn update(message: Message) -> Action` function, returning e.g.
//! `Action::operation(Operation::Back)`.
//!
//! It is the responsibility of the ancestor component to handle the `Operation`
//! that is returned from that child view. To make the code easier to follow,
//! it may be advantageous to define a separate `fn perform(operation: Operation) -> Task`
//! function to handle any operations returned by the child view. In some cases,
//! those operations may result in yet another [`Task`], which would require
//! the parent component to chain the tasks together. An example of this can be
//! be seen in the `fn update` function in `src/main.rs`.
//!
//! This design pattern is common in many [`iced`] applications, although the exact
//! implementation may vary. It is often the case that the `Action` is simply
//! an enum which contains either operations or tasks, but our proposed design
//! allows for more flexibility and clarity at the expense of slightly more
//! boilerplate upfront, which we think is a worthy tradeoff.
use iced::advanced::graphics::futures::MaybeSend;
use iced::Task;
use std::fmt;

pub struct Action<Operation, Message> {
    pub operation: Option<Operation>,
    pub task: Task<Message>,
}

impl<Operation, Message> Action<Operation, Message> {
    /// Create a new `Action` with no `Operation` or [`Task`](iced::Task).
    pub fn none() -> Self {
        Self {
            operation: None,
            task: Task::none(),
        }
    }

    /// Create a new `Action` with an `Operation` and a [`Task`](iced::Task).
    pub fn new(operation: Operation, task: Task<Message>) -> Self {
        Self {
            operation: Some(operation),
            task,
        }
    }

    /// Create a new `Action` with an `Operation` to be handled by some ancestor
    /// component.
    pub fn operation(operation: Operation) -> Self {
        Self {
            operation: Some(operation),
            task: Task::none(),
        }
    }

    /// Create a new `Action` with a [`Task`](iced::Task).
    pub fn task(task: Task<Message>) -> Self {
        Self {
            operation: None,
            task,
        }
    }

    /// Map the message of the `Action`'s [`Task`](iced::Task) to a different type.
    pub fn map<N>(self, f: impl Fn(Message) -> N + MaybeSend + 'static) -> Action<Operation, N>
    where
        Message: MaybeSend + 'static,
        N: MaybeSend + 'static,
    {
        Action {
            operation: self.operation,
            task: self.task.map(f),
        }
    }

    /// Maps the `Operation` of the `Action` to a different type.
    pub fn map_operation<N>(
        self,
        f: impl Fn(Operation) -> N + MaybeSend + 'static,
    ) -> Action<N, Message>
    where
        Operation: MaybeSend + 'static,
        N: MaybeSend + 'static,
    {
        Action {
            operation: self.operation.map(f),
            task: self.task,
        }
    }

    /// Sets the `Operation` of an `Action`.
    pub fn with_operation(mut self, operation: Operation) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Sets the [`Task`](iced::Task) of an `Action`.
    pub fn with_task(mut self, task: Task<Message>) -> Self {
        self.task = task;
        self
    }
}

impl<Operation: fmt::Debug, Message> fmt::Debug for Action<Operation, Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("operation", &self.operation)
            .finish()
    }
}
