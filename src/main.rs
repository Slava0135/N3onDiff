mod feedback;
mod input;
mod objective;
mod observer;
mod output;

use std::{env, path::PathBuf};

use base64::prelude::*;
use clap::Parser;
use feedback::{go_cover::GoCoverFeedback, type_state::TypeStateFeedback};
use input::ByteCodeInput;
use libafl::prelude::*;
use libafl_bolts::{
    core_affinity::Cores,
    current_nanos,
    prelude::CoreId,
    rands::StdRand,
    shmem::{ShMemProvider, StdShMemProvider},
    tuples::{tuple_list, Handled},
};
use observer::GoCoverObserver;
use rand::seq::{IteratorRandom, SliceRandom};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Report error if VMs exit with different status. Can result in many false positives!",
        name = "DETECT_STATUS_DIFFERENCE",
        default_value_t = false
    )]
    detect_status_diff: std::primitive::bool,

    #[arg(
        short,
        long,
        help = "Read initial corpus from file",
        name = "READ_CORPUS_FROM_FILE",
        default_value_t = true
    )]
    read_corpus_from_file: std::primitive::bool,

    #[arg(
        short,
        long,
        help = "Spread inputs from initial corpus between cores",
        name = "SPREAD_CORPUS",
        default_value_t = true
    )]
    spread_corpus: std::primitive::bool,

    #[arg(
        short = 'p',
        long,
        help = "Choose the broker TCP port",
        name = "PORT",
        default_value = "7777"
    )]
    broker_port: u16,

    #[arg(
        short,
        long,
        value_parser = Cores::from_cmdline,
        help = "Spawn a client in each of the provided cores. Broker runs in the 0th core. 'all' to select all available cores. eg: '1,2-4,6' selects the cores 1,2,3,4,6.",
        name = "CORES"
    )]
    cores: Cores,
}

fn main() {
    let mut rng = rand::thread_rng();
    let args = Args::parse();

    let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");
    let monitor = MultiMonitor::new(|s| println!("{s}"));

    let temp_dir = env::temp_dir().join("N3onDiff");
    std::fs::create_dir(temp_dir.as_path()).unwrap_or(());

    let mut corpus_from_file = Vec::new();
    if args.read_corpus_from_file {
        for line in std::fs::read_to_string("./corpus/corpus.txt")
            .unwrap()
            .lines()
        {
            corpus_from_file.push(Testcase::new(ByteCodeInput {
                opcodes: BASE64_STANDARD.decode(line).unwrap(),
            }));
        }
    }
    corpus_from_file.shuffle(&mut rng);
    let testcases_chunks: Vec<Vec<_>> = corpus_from_file
        .chunks(args.cores.ids.len())
        .map(|x| x.to_vec())
        .collect();
    let mut core_n = 0;

    let mut run_client = |_state: Option<_>, mut restarting_mgr, core_id: CoreId| {
        let neogo_stdout_observer = StdOutObserver::new("neogo-stdout-observer");
        let neosharp_stdout_observer = StdOutObserver::new("neosharp-stdout-observer");

        let mut objective = objective::DiffStdOutObjective::new(
            neogo_stdout_observer.handle(),
            neosharp_stdout_observer.handle(),
            args.detect_status_diff,
        );

        let core_temp_dir = temp_dir.join(core_id.0.to_string());
        std::fs::create_dir(core_temp_dir.as_path()).unwrap_or(());

        let go_cover_dir = core_temp_dir.join("go-cover");
        std::fs::create_dir(go_cover_dir.as_path()).unwrap_or(());
        let go_cover_observer = GoCoverObserver::new(go_cover_dir.clone().into_boxed_path());

        let mut feedback = feedback_or!(
            TypeStateFeedback::new(vec![
                neogo_stdout_observer.handle(),
                neosharp_stdout_observer.handle(),
            ]),
            GoCoverFeedback::new(go_cover_observer.handle())
        );

        let neogo_executor = CommandExecutor::builder()
            .program("./harness/neo-go")
            .env("GOCOVERDIR", go_cover_dir.as_path())
            .arg_input_arg()
            .arg("DUMMY")
            .stdout_observer(neogo_stdout_observer.handle())
            .build(tuple_list!(neogo_stdout_observer, go_cover_observer))
            .unwrap();

        let neosharp_executor = CommandExecutor::builder()
            .program("./harness/neo-sharp")
            .arg_input_arg()
            .arg("DUMMY")
            .stdout_observer(neosharp_stdout_observer.handle())
            .build(tuple_list!(neosharp_stdout_observer))
            .unwrap();

        let mut executor = DiffExecutor::new(neogo_executor, neosharp_executor, ());

        let mut state = StdState::new(
            StdRand::with_seed(current_nanos()),
            InMemoryCorpus::new(),
            OnDiskCorpus::new(PathBuf::from("./crashes")).unwrap(),
            &mut feedback,
            &mut objective,
        )
        .unwrap();

        let corpus = state.corpus_mut();

        corpus
            .add(Testcase::new(ByteCodeInput {
                opcodes: BASE64_STANDARD.decode("DAxIZWxsbyB3b3JsZCE=").unwrap(),
            }))
            .unwrap();

        if args.spread_corpus {
            for tc in testcases_chunks[core_n].clone() {
                corpus.add(tc).unwrap();
            }
        } else {
            for tc in corpus_from_file.iter() {
                corpus.add(tc.clone()).unwrap();
            }
        }
        core_n += 1;

        let scheduler = QueueScheduler::new();

        let mutator = StdScheduledMutator::new(havoc_mutations());
        let mut stages = tuple_list!(StdMutationalStage::new(mutator));

        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        loop {
            match fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut restarting_mgr) {
                Ok(_) => break,
                Err(Error::ShuttingDown) => break,
                Err(err) => println!("{err:?}"),
            }
        }
        Ok(())
    };

    match Launcher::builder()
        .shmem_provider(shmem_provider)
        .configuration(EventConfig::from_name("default"))
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&args.cores)
        .broker_port(args.broker_port)
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {err:?}"),
    }
}
