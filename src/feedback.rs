use std::{borrow::Cow, collections::HashSet};

use libafl::{prelude::*, state::State};
use libafl_bolts::{
    tuples::{Handle, MatchNameRef},
    Named,
};

use crate::output::{parse, StackItem};

#[derive(Clone)]
pub struct TypeStateFeedback {
    stdout_observers: Vec<Handle<StdOutObserver>>,
    states: HashSet<TypeState>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TypeState {
    opcode: u8,
    fst_type: String,
    snd_type: String
}

impl TypeStateFeedback {
    pub fn new(stdout_observers: Vec<Handle<StdOutObserver>>) -> Self {
        Self {
            stdout_observers: stdout_observers,
            states: HashSet::new()
        }
    }
}

impl<S> Feedback<S> for TypeStateFeedback
where
    S: State,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &<S>::Input,
        observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        let mut new_state_found = false;
        for obs in &self.stdout_observers {
            let out = observers
                .get(obs)
                .unwrap()
                .stdout
                .as_ref()
                .expect("no output found (first)")
                .clone();
            let out = parse(&out);
            let ts = match &out.estack[..] {
                [fst] => {
                    TypeState {
                        opcode: out.lastop,
                        fst_type: fst.itype.clone(),
                        snd_type: String::new(),
                    }
                },
                [fst, snd] => {
                    TypeState {
                        opcode: out.lastop,
                        fst_type: fst.itype.clone(),
                        snd_type: snd.itype.clone(),
                    }
                },
                [fst,  snd, ..] => {
                    TypeState {
                        opcode: out.lastop,
                        fst_type: fst.itype.clone(),
                        snd_type: snd.itype.clone(),
                    }
                },
                _ => continue
            };
            if self.states.insert(ts) {
                new_state_found = true;
            }
        }
        Ok(new_state_found)
    }
}

impl Named for TypeStateFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("TypeStateFeedback")
    }
}
