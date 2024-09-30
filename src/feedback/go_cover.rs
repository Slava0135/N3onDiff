use std::{borrow::Cow, collections::HashSet};

use libafl::{
    events::EventFirer,
    prelude::{Feedback, ObserversTuple},
    state::State,
};
use libafl_bolts::{
    tuples::{Handle, MatchNameRef},
    Named,
};

use crate::observer::GoCoverObserver;

pub struct GoCoverFeedback {
    go_cover_observer: Handle<GoCoverObserver>,
    all_coverage: HashSet<String>,
}

impl GoCoverFeedback {
    pub fn new(go_cover_observer: Handle<GoCoverObserver>) -> Self {
        Self {
            go_cover_observer: go_cover_observer,
            all_coverage: HashSet::new(),
        }
    }
}

impl<S> Feedback<S> for GoCoverFeedback
where
    S: State,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &<S>::Input,
        observers: &OT,
        _exit_kind: &libafl::prelude::ExitKind,
    ) -> Result<bool, libafl::Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        let coverage = &observers
            .get(&self.go_cover_observer)
            .expect("failed to read coverage")
            .coverage;
        let c = self.all_coverage.clone();
        let diff: Vec<&String> = coverage.difference(&c).collect();
        if diff.is_empty() {
            Ok(false)
        } else {
            for v in diff {
                self.all_coverage.insert(v.clone());
            }
            Ok(true)
        }
    }
}

impl Named for GoCoverFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("GoCoverFeedback")
    }
}
