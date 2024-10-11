use std::borrow::Cow;

use base64::{
    prelude::{BASE64_STANDARD, BASE64_URL_SAFE},
    Engine,
};
use libafl::{prelude::*, state::State};
use libafl_bolts::{
    impl_serdeany,
    tuples::{Handle, MatchNameRef},
    Named,
};
use serde::{Deserialize, Serialize};

use crate::output::{parse, Output};

#[derive(Clone)]
pub struct DiffStdOutObjective {
    pub fst_stdout_observer: Handle<StdOutObserver>,
    pub snd_stdout_observer: Handle<StdOutObserver>,
    diff_std_out_metadata: DiffStdOutMetadata,
    detect_status_diff: bool,
}

impl DiffStdOutObjective {
    pub fn new(
        fst_stdout_observer: Handle<StdOutObserver>,
        snd_stdout_observer: Handle<StdOutObserver>,
        detect_status_diff: bool,
    ) -> DiffStdOutObjective {
        DiffStdOutObjective {
            fst_stdout_observer: fst_stdout_observer,
            snd_stdout_observer: snd_stdout_observer,
            diff_std_out_metadata: DiffStdOutMetadata::default(),
            detect_status_diff: detect_status_diff,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct DiffStdOutMetadata {
    base64: Option<String>,
    fst: Option<Output>,
    snd: Option<Output>,
    cause: Option<String>,
}

impl_serdeany!(DiffStdOutMetadata);

impl<S> Feedback<S> for DiffStdOutObjective
where
    S: State,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        input: &<S>::Input,
        observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        self.diff_std_out_metadata = DiffStdOutMetadata::default();
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
        match (parse(&fst_out), parse(&snd_out)) {
            (Some(fst_out), Some(snd_out)) => {
                if fst_out.status != snd_out.status {
                    self.diff_std_out_metadata = DiffStdOutMetadata {
                        base64: Some(as_standard_base64(input.generate_name(None))),
                        fst: Some(fst_out),
                        snd: Some(snd_out),
                        cause: Some(String::from("different status")),
                    };
                    Ok(self.detect_status_diff)
                } else {
                    match fst_out.status.as_str() {
                        "VM halted" => {
                            if fst_out.estack != snd_out.estack {
                                self.diff_std_out_metadata = DiffStdOutMetadata {
                                    base64: Some(as_standard_base64(input.generate_name(None))),
                                    fst: Some(fst_out),
                                    snd: Some(snd_out),
                                    cause: Some(String::from("different stack")),
                                };
                                Ok(true)
                            } else {
                                Ok(false)
                            }
                        }
                        _ => Ok(false),
                    }
                }
            }
            (None, None) => Ok(false),
            _ => Ok(false),
        }
    }

    fn append_metadata<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _observers: &OT,
        testcase: &mut Testcase<<S>::Input>,
    ) -> Result<(), Error>
    where
        OT: ObserversTuple<S>,
        EM: EventFirer<State = S>,
    {
        testcase
            .metadata_map_mut()
            .insert(self.diff_std_out_metadata.clone());
        Ok(())
    }
}

impl Named for DiffStdOutObjective {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("DiffStdOutObjective")
    }
}

// idk how you do this with types instead
fn as_standard_base64(s: String) -> String {
    BASE64_STANDARD.encode(BASE64_URL_SAFE.decode(s).expect("expected base64 url"))
}
