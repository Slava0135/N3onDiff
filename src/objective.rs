use std::borrow::Cow;

use libafl::{prelude::*, state::State};
use libafl_bolts::{
    tuples::{Handle, MatchNameRef},
    Named,
};

use crate::output::parse;

#[derive(Clone)]
pub struct DiffStdOutObjective {
    pub fst_stdout_observer: Handle<StdOutObserver>,
    pub snd_stdout_observer: Handle<StdOutObserver>,
}

impl<S> Feedback<S> for DiffStdOutObjective
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
        let fst_out = observers
            .get(&self.fst_stdout_observer)
            .unwrap()
            .stdout
            .as_ref()
            .expect("no output found (first)")
            .clone();
        let snd_out = observers
            .get(&self.snd_stdout_observer)
            .unwrap()
            .stdout
            .as_ref()
            .expect("no output found (second)")
            .clone();
        let fst_out = parse(&fst_out);
        let snd_out = parse(&snd_out);
        if fst_out.status != snd_out.status {
            return Ok(true);
        }
        if fst_out.estack != snd_out.estack {
            return Ok(true);
        }
        Ok(false)
    }
}

impl Named for DiffStdOutObjective {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("DiffStdOutObjective")
    }
}
