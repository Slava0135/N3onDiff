use std::borrow::Cow;

use libafl::{prelude::*, state::State};
use libafl_bolts::{
    tuples::{Handle, MatchNameRef},
    Named,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Output {
    status: String,
    errmsg: String,
    estack: Value,
}

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
            .clone();
        let snd_out = observers
            .get(&self.snd_stdout_observer)
            .unwrap()
            .stdout
            .clone();
        match (fst_out, snd_out) {
            (Some(fst_out), Some(snd_out)) => {
                let fst_out: Output = serde_json::from_slice(&fst_out)
                    .expect("failed to read json output from stdout (first)");
                let snd_out: Output = serde_json::from_slice(&snd_out)
                    .expect("failed to read json output from stdout (second)");
                if fst_out.status != snd_out.status {
                    return Ok(true);
                }
                if fst_out.estack != snd_out.estack {
                    return Ok(true);
                }
                Ok(false)
            }
            _ => panic!("no output found"),
        }
    }
}

impl Named for DiffStdOutObjective {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("DiffStdOutObjective")
    }
}
