// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use std::slice::Iter;

#[derive(Debug, PartialEq)]
pub(crate) struct State<B, L, E> {
    transition: Transition<B, L, E>,
}

pub(crate) type ParseResult<B, L, E> = Result<State<B, L, E>, E>;

type Transition<B, L, E> = fn(b: &mut B, l: L) -> ParseResult<B, L, E>;
pub(crate) type Termination<B, L, E> = fn(last_state: State<B, L, E>, b: &mut B) -> Result<(), E>;

impl<B, L, E> State<B, L, E> {
    pub(crate) fn next(&self, b: &mut B, l: L) -> ParseResult<B, L, E> {
        let transition = self.transition;
        transition(b, l)
    }
}

impl<B, L, E> Clone for State<B, L, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B, L, E> Copy for State<B, L, E> {}

pub(crate) const fn state<B, L, E>(transition: Transition<B, L, E>) -> State<B, L, E> {
    State { transition }
}

pub(crate) struct StateMachine<B, L, E> {
    start_state: State<B, L, E>,
    termination: Termination<B, L, E>,
}

impl<B, L, E> StateMachine<B, L, E>
where
    L: Copy,
{
    pub(crate) fn run(&self, b: &mut B, l: Iter<L>) -> Result<(), E> {
        let mut state = self.start_state;
        for next in l {
            state = state.next(b, *next)?;
        }
        (self.termination)(state, b)
    }
}
pub(crate) const fn state_machine<B, L, E>(
    start_state: State<B, L, E>,
    termination: Termination<B, L, E>,
) -> StateMachine<B, L, E> {
    StateMachine {
        start_state,
        termination,
    }
}
