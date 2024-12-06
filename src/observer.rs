use std::{
    borrow::Cow,
    collections::HashSet,
    fs::{self, read_to_string},
    path::Path,
    process::Command,
};

use libafl::{inputs::UsesInput, prelude::Observer};
use libafl_bolts::Named;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GoCoverObserver {
    pub coverage: HashSet<String>,
    cover_dir: Box<Path>,
    cover_merged_dir: Box<Path>,
    cover_tmp_dir: Box<Path>,
    profile_path: Box<Path>,
}

impl GoCoverObserver {
    pub fn new(
        cover_dir: Box<Path>,
        cover_merged_dir: Box<Path>,
        cover_tmp_dir: Box<Path>,
    ) -> GoCoverObserver {
        let mut profile_path = cover_dir.clone().to_path_buf();
        profile_path.push("profile.txt");
        GoCoverObserver {
            coverage: HashSet::new(),
            cover_dir,
            cover_merged_dir,
            cover_tmp_dir,
            profile_path: profile_path.into_boxed_path(),
        }
    }
}

impl<S> Observer<S> for GoCoverObserver
where
    S: UsesInput,
{
    fn flush(&mut self) -> Result<(), libafl::Error> {
        Ok(())
    }

    fn pre_exec(
        &mut self,
        _state: &mut S,
        _input: &<S as UsesInput>::Input,
    ) -> Result<(), libafl::Error> {
        for entry in std::fs::read_dir(self.cover_dir.as_ref())? {
            let entry = entry?;
            std::fs::remove_file(entry.path())?;
        }
        Ok(())
    }

    fn post_exec(
        &mut self,
        _state: &mut S,
        _input: &<S as UsesInput>::Input,
        _exit_kind: &libafl::prelude::ExitKind,
    ) -> Result<(), libafl::Error> {
        self.coverage.clear();
        let mut cover_cmd = Command::new("go");
        cover_cmd
            .arg("tool")
            .arg("covdata")
            .arg("textfmt")
            .arg("-i")
            .arg(self.cover_dir.as_ref())
            .arg("-o")
            .arg(self.profile_path.as_ref())
            .arg("-pkg")
            .arg("github.com/nspcc-dev/neo-go/pkg/vm");
        cover_cmd.output()?;
        for line in read_to_string(self.profile_path.as_ref())?.lines() {
            if line.starts_with("mode") {
                continue;
            }
            let data: Vec<_> = line.split(' ').collect();
            let location = data[0];
            let count: i32 = data[2].parse().unwrap();
            if count != 0 {
                self.coverage.insert(String::from(location));
            }
        }
        for entry in std::fs::read_dir(self.cover_tmp_dir.as_ref())? {
            let entry = entry?;
            std::fs::remove_file(entry.path())?;
        }
        let mut cover_merge_cmd = Command::new("go");
        cover_merge_cmd
            .arg("tool")
            .arg("covdata")
            .arg("merge")
            .arg(format!(
                "-i={},{}",
                self.cover_dir.display(),
                self.cover_merged_dir.display()
            ))
            .arg("-o")
            .arg(self.cover_tmp_dir.as_ref());
        cover_merge_cmd.output()?;
        for entry in std::fs::read_dir(self.cover_merged_dir.as_ref())? {
            let entry = entry?;
            std::fs::remove_file(entry.path())?;
        }
        for entry in std::fs::read_dir(self.cover_tmp_dir.as_ref())? {
            let entry = entry?;
            fs::copy(
                entry.path(),
                self.cover_merged_dir.as_ref().join(entry.file_name()),
            )?;
        }
        Ok(())
    }

    fn pre_exec_child(
        &mut self,
        _state: &mut S,
        _input: &<S as UsesInput>::Input,
    ) -> Result<(), libafl::Error> {
        Ok(())
    }

    fn post_exec_child(
        &mut self,
        _state: &mut S,
        _input: &<S as UsesInput>::Input,
        _exit_kind: &libafl::prelude::ExitKind,
    ) -> Result<(), libafl::Error> {
        Ok(())
    }
}

impl Named for GoCoverObserver {
    fn name(&self) -> &std::borrow::Cow<'static, str> {
        &Cow::Borrowed("GoCoverObserver")
    }
}
