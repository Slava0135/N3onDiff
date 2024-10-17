use std::borrow::Cow;

use libafl::{prelude::*, state::State};
use libafl_bolts::{
    impl_serdeany,
    tuples::{Handle, MatchNameRef},
    Named,
};
use serde::{Deserialize, Serialize};

use crate::{
    input::ByteCodeInput,
    output::{parse, Output},
};

#[derive(Clone)]
pub struct DiffStdOutObjective {
    pub fst_stdout_observer: Handle<StdOutObserver>,
    pub snd_stdout_observer: Handle<StdOutObserver>,
    diff_std_out_metadata: DiffStdOutMetadata,
    detect_status_diff: bool,
    detect_crash_diff: bool,
}

impl DiffStdOutObjective {
    pub fn new(
        fst_stdout_observer: Handle<StdOutObserver>,
        snd_stdout_observer: Handle<StdOutObserver>,
        detect_status_diff: bool,
        detect_crash_diff: bool,
    ) -> DiffStdOutObjective {
        DiffStdOutObjective {
            fst_stdout_observer: fst_stdout_observer,
            snd_stdout_observer: snd_stdout_observer,
            diff_std_out_metadata: DiffStdOutMetadata::default(),
            detect_status_diff: detect_status_diff,
            detect_crash_diff: detect_crash_diff,
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
    S: State + UsesInput<Input = ByteCodeInput>,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        input: &<S>::Input,
        observers: &OT,
        exit_kind: &ExitKind,
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
        match exit_kind {
            ExitKind::Diff { primary, secondary }
                if self.detect_crash_diff
                    && (primary.to_owned() == DiffExitKind::Crash
                        || secondary.to_owned() == DiffExitKind::Crash) =>
            {
                self.diff_std_out_metadata = DiffStdOutMetadata {
                    base64: Some(input.as_standard_base64()),
                    fst: parse(&fst_out),
                    snd: parse(&snd_out),
                    cause: Some(
                        String::from("different exit code: ")
                            + &serde_json::to_string(primary).unwrap().replace("\"", "")
                            + " / "
                            + &serde_json::to_string(secondary).unwrap().replace("\"", ""),
                    ),
                };
                return Ok(true);
            }
            _ => (),
        }
        match (parse(&fst_out), parse(&snd_out)) {
            (Some(fst_out), Some(snd_out)) if fst_out.status == snd_out.status => {
                match fst_out.status.as_str() {
                    "VM halted" => {
                        if fst_out.estack != snd_out.estack {
                            self.diff_std_out_metadata = DiffStdOutMetadata {
                                base64: Some(input.as_standard_base64()),
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
            (Some(fst_out), Some(snd_out))
                if fst_out.status != snd_out.status && self.detect_status_diff =>
            {
                self.diff_std_out_metadata = DiffStdOutMetadata {
                    base64: Some(input.as_standard_base64()),
                    fst: Some(fst_out),
                    snd: Some(snd_out),
                    cause: Some(String::from("different status")),
                };
                Ok(true)
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
