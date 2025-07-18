use std::{marker::PhantomData, mem};

use crate::span::Spanned;

mod example;

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TokenType {
    Value,
    Precedence {
        precedence: i8,
        associativity: Associativity,
    },
}

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Associativity {
    /// Left to right
    Left,
    /// Right to left
    Right,
    /// Expects to be closed by a token on the right
    ClosedRight,
}

pub trait IsResolvedToken {
    fn get_type(&self) -> TokenType;
}

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OrderingBehaviour {
    Right { precedence: i8, closed: bool },
    SoftLeft { precedence: i8 },
    ClosedLeft,
}

pub trait IsOrdering {
    fn behaviour(&self) -> OrderingBehaviour;
}

pub enum StackEntry<T: IsResolvedToken, O: IsOrdering> {
    Resolved(Spanned<T>),
    Ordering(Spanned<O>),
}

pub trait FromStackEntry: Sized {
    type Token: IsResolvedToken;
    type Ordering: IsOrdering;

    fn from_entry(token: &StackEntry<Self::Token, Self::Ordering>) -> Self;
}

pub type StackEntryToken<S> = <S as FromStackEntry>::Token;
pub type StackEntryOrdering<S> = <S as FromStackEntry>::Ordering;
pub type StackEntryFrom<S> = StackEntry<StackEntryToken<S>, StackEntryOrdering<S>>;

pub trait HasStateTransition<T: ?Sized> {
    type Token: IsResolvedToken;
    type Ordering: IsOrdering;
    type Error;

    #[allow(clippy::type_complexity)]
    fn transition(
        self,
        token: T,
    ) -> Result<StackEntry<Self::Token, Self::Ordering>, Spanned<Self::Error>>;
}

pub trait IsState<T: IsResolvedToken, O: IsOrdering> {
    type Error;

    fn update(&mut self, token: &StackEntry<T, O>);

    fn proccess_closed(&mut self, token: &mut Spanned<T>);

    fn delete_closed_ordering(&mut self, ordering: Spanned<O>);

    fn no_ordering_found(&self) -> Spanned<Self::Error>;
}
impl<T: IsResolvedToken, O: IsOrdering> IsState<T, O> for () {
    type Error = ();

    #[inline(always)]
    fn update(&mut self, _: &StackEntry<T, O>) {}

    #[inline(always)]
    fn proccess_closed(&mut self, _: &mut Spanned<T>) {}

    #[inline(always)]
    fn delete_closed_ordering(&mut self, _: Spanned<O>) {}

    #[inline(always)]
    fn no_ordering_found(&self) -> Spanned<Self::Error> {
        Spanned::default()
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProcessTokenIteratorState<T: IsResolvedToken, O: IsOrdering> {
    Pending,
    ProcessResolved(Spanned<T>),
    ProcessOrdering(Spanned<O>),
    ClearingStack,
    Completed,
}

pub struct CreateTokenProcessor<
    T,
    TS: FromStackEntry
        + HasStateTransition<T, Token = StackEntryToken<TS>, Ordering = StackEntryOrdering<TS>>
        + Default,
    S: IsState<StackEntryToken<TS>, StackEntryOrdering<TS>>,
> {
    __raw_token: PhantomData<T>,
    __tree_state: PhantomData<TS>,
    __state: PhantomData<S>,
}

pub struct ProcessTokenIterator<
    T,
    TS: FromStackEntry
        + HasStateTransition<T, Token = StackEntryToken<TS>, Ordering = StackEntryOrdering<TS>>
        + Default,
    S: IsState<StackEntryToken<TS>, StackEntryOrdering<TS>> + Default,
    I: Iterator<Item = T>,
> {
    internal_state: ProcessTokenIteratorState<StackEntryToken<TS>, StackEntryOrdering<TS>>,
    tokens: I,
    tree_state: TS,
    state: S,
    stack: Vec<StackEntryFrom<TS>>,
}

impl<
    T,
    TS: FromStackEntry
        + HasStateTransition<T, Token = StackEntryToken<TS>, Ordering = StackEntryOrdering<TS>>
        + Default,
    S: IsState<StackEntryToken<TS>, StackEntryOrdering<TS>, Error = TS::Error> + Default,
> CreateTokenProcessor<T, TS, S>
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        iter: impl Iterator<Item = T>,
    ) -> ProcessTokenIterator<T, TS, S, impl Iterator<Item = T>> {
        ProcessTokenIterator {
            internal_state: ProcessTokenIteratorState::Pending,
            tokens: iter,
            tree_state: TS::default(),
            state: S::default(),
            stack: Vec::new(),
        }
    }
}

impl<
    T,
    TS: FromStackEntry
        + HasStateTransition<T, Token = StackEntryToken<TS>, Ordering = StackEntryOrdering<TS>>
        + Default,
    S: IsState<StackEntryToken<TS>, StackEntryOrdering<TS>, Error = TS::Error> + Default,
    I: Iterator<Item = T>,
> Iterator for ProcessTokenIterator<T, TS, S, I>
{
    type Item = Result<Spanned<StackEntryToken<TS>>, Spanned<TS::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        'main: loop {
            match self.internal_state {
                ProcessTokenIteratorState::Pending => {
                    let Some(token) = self.tokens.next() else {
                        self.internal_state = ProcessTokenIteratorState::ClearingStack;
                        continue 'main;
                    };

                    let entry = match mem::take(&mut self.tree_state).transition(token) {
                        Ok(entry) => entry,
                        Err(error) => {
                            self.internal_state = ProcessTokenIteratorState::Completed;
                            return Some(Err(error));
                        }
                    };

                    self.tree_state = TS::from_entry(&entry);
                    self.state.update(&entry);

                    self.internal_state = match entry {
                        StackEntry::Resolved(resolved) => {
                            ProcessTokenIteratorState::ProcessResolved(resolved)
                        }
                        StackEntry::Ordering(ordering) => {
                            ProcessTokenIteratorState::ProcessOrdering(ordering)
                        }
                    };

                    continue 'main;
                }
                ProcessTokenIteratorState::ProcessResolved(ref resolved) => {
                    match resolved.inner.get_type() {
                        TokenType::Value => {
                            let resolved = match mem::replace(
                                &mut self.internal_state,
                                ProcessTokenIteratorState::Pending,
                            ) {
                                ProcessTokenIteratorState::ProcessResolved(resolved) => resolved,
                                _ => unreachable!(),
                            };

                            return Some(Ok(resolved));
                        }
                        TokenType::Precedence {
                            precedence,
                            associativity,
                        } => {
                            let result = self.stack.pop_if(|token| {
                                let last_precedence = match token {
                                    StackEntry::Resolved(resolved) => {
                                        match resolved.inner.get_type() {
                                            TokenType::Precedence {
                                                precedence: last_precedence,
                                                associativity,
                                            } => match associativity {
                                                Associativity::Left | Associativity::Right => {
                                                    last_precedence
                                                }
                                                Associativity::ClosedRight => return false,
                                            },
                                            _ => unreachable!(
                                                "Stack should only contain precedence tokens!"
                                            ),
                                        }
                                    }
                                    StackEntry::Ordering(ordering) => {
                                        match ordering.inner.behaviour() {
                                            OrderingBehaviour::Right {
                                                precedence,
                                                closed: false,
                                            } => precedence,
                                            _ => return false,
                                        }
                                    }
                                };

                                last_precedence >= precedence
                                    && (associativity == Associativity::Left
                                        || precedence != last_precedence)
                            });

                            if let Some(StackEntry::Resolved(resolved)) = result {
                                return Some(Ok(resolved));
                            }

                            let resolved = match mem::replace(
                                &mut self.internal_state,
                                ProcessTokenIteratorState::Pending,
                            ) {
                                ProcessTokenIteratorState::ProcessResolved(resolved) => resolved,
                                _ => unreachable!(),
                            };

                            self.stack.push(StackEntry::Resolved(resolved));
                        }
                    }
                }
                ProcessTokenIteratorState::ProcessOrdering(ref ordering) => {
                    match ordering.inner.behaviour() {
                        OrderingBehaviour::Right { precedence, .. } => {
                            let result = self.stack.pop_if(|token| {
                                let last_precedence = match token {
                                    StackEntry::Resolved(resolved) => {
                                        match resolved.inner.get_type() {
                                            TokenType::Precedence {
                                                precedence: last_precedence,
                                                associativity,
                                            } => match associativity {
                                                Associativity::Left | Associativity::Right => {
                                                    last_precedence
                                                }
                                                Associativity::ClosedRight => return false,
                                            },
                                            _ => unreachable!(
                                                "Stack should only contain precedence tokens!"
                                            ),
                                        }
                                    }
                                    StackEntry::Ordering(ordering) => {
                                        match ordering.inner.behaviour() {
                                            OrderingBehaviour::Right {
                                                precedence,
                                                closed: false,
                                            } => precedence,
                                            _ => return false,
                                        }
                                    }
                                };

                                last_precedence > precedence
                            });

                            if let Some(StackEntry::Resolved(resolved)) = result {
                                return Some(Ok(resolved));
                            }

                            let ordering = match mem::replace(
                                &mut self.internal_state,
                                ProcessTokenIteratorState::Pending,
                            ) {
                                ProcessTokenIteratorState::ProcessOrdering(ordering) => ordering,
                                _ => unreachable!(),
                            };

                            self.stack.push(StackEntry::Ordering(ordering));
                        }
                        OrderingBehaviour::SoftLeft { precedence } => {
                            let result = self.stack.pop_if(|token| {
                                let last_precedence = match token {
                                    StackEntry::Resolved(resolved) => {
                                        match resolved.inner.get_type() {
                                            TokenType::Precedence {
                                                precedence: last_precedence,
                                                associativity,
                                            } => match associativity {
                                                Associativity::Left | Associativity::Right => {
                                                    last_precedence
                                                }
                                                Associativity::ClosedRight => return false,
                                            },
                                            _ => unreachable!(
                                                "Stack should only contain precedence tokens!"
                                            ),
                                        }
                                    }
                                    StackEntry::Ordering(ordering) => {
                                        match ordering.inner.behaviour() {
                                            OrderingBehaviour::Right {
                                                precedence,
                                                closed: false,
                                            } => precedence,
                                            _ => return false,
                                        }
                                    }
                                };

                                last_precedence >= precedence
                            });

                            if let Some(StackEntry::Resolved(resolved)) = result {
                                return Some(Ok(resolved));
                            }

                            self.internal_state = ProcessTokenIteratorState::Pending;
                        }
                        OrderingBehaviour::ClosedLeft => {
                            if let Some(token) = self.stack.pop() {
                                match token {
                                    StackEntry::Resolved(mut resolved) => {
                                        match resolved.inner.get_type() {
                                            TokenType::Precedence {
                                                precedence: _,
                                                associativity: Associativity::ClosedRight,
                                            } => {
                                                self.internal_state =
                                                    ProcessTokenIteratorState::Pending;
                                                self.state.proccess_closed(&mut resolved);
                                            }
                                            _ => return Some(Ok(resolved)),
                                        }
                                        return Some(Ok(resolved));
                                    }
                                    StackEntry::Ordering(last_ordering) => {
                                        match last_ordering.inner.behaviour() {
                                            OrderingBehaviour::Right {
                                                precedence: _,
                                                closed: true,
                                            } => {
                                                self.internal_state =
                                                    ProcessTokenIteratorState::Pending;
                                                self.state.delete_closed_ordering(last_ordering);
                                                continue 'main;
                                            }
                                            _ => continue 'main,
                                        }
                                    }
                                }
                            } else {
                                self.internal_state = ProcessTokenIteratorState::Completed;
                                return Some(Err(self.state.no_ordering_found()));
                            }
                        }
                    }
                }
                ProcessTokenIteratorState::ClearingStack => {
                    while let Some(entry) = self.stack.pop() {
                        match entry {
                            StackEntry::Resolved(resolved) => return Some(Ok(resolved)),
                            StackEntry::Ordering(ordering) => {
                                self.state.delete_closed_ordering(ordering)
                            }
                        }
                    }

                    self.internal_state = ProcessTokenIteratorState::Completed;
                    return None;
                }
                ProcessTokenIteratorState::Completed => return None,
            }
        }
    }
}
