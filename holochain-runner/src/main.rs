use embedded_runner::{async_main, HcConfig};
use emit::StateSignal;
use std::{fs::read, process::exit};
use structopt::StructOpt;

mod config;
mod embedded_runner;
mod emit;
mod generate_key;
mod install_enable;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "holochain-runner",
    about = "wrapped Holochain Conductor with Status Update events, and a good SIGTERM kill switch "
)]
struct Opt {
    #[structopt(
        default_value = "databases",
        help = "configuration values for `app_id` and `app_ws_port`
will be overridden if an existing
configuration is found at this path"
    )]
    datastore_path: String,

    #[structopt(long, default_value = "main-app")]
    app_id: String,

    #[structopt(long, default_value = "8888")]
    app_ws_port: u16,

    #[structopt(long, default_value = "1234")]
    admin_ws_port: u16,

    #[structopt(long, default_value = "keystore")]
    keystore_path: String,

    #[structopt(long)]
    membrane_proof: Option<String>,

    // community
    #[structopt(
        long,
        default_value = "kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--"
    )]
    proxy_url: String,
}

fn main() {
    // Create the runtime
    // we want to use multiple threads
    let rt = tokio::runtime::Builder::new_multi_thread()
        // we use both IO and Time tokio utilities
        .enable_all()
        // give our threads a descriptive name (they'll be numbered too)
        .thread_name("acorn-tokio-thread")
        // build the runtime
        .build()
        // panic if we cannot (we cannot run without it)
        .expect("can build tokio runtime");
    let _guard = rt.enter();

    // set up the ctrlc shutdown listener
    // listening for SIGINT or SIGTERM (unix), just CTRC-C on windows
    let rt_handle = rt.handle().clone();
    let (shutdown_sender, mut shutdown_receiver) = tokio::sync::mpsc::channel::<bool>(1);
    ctrlc::set_handler(move || {
        println!("ctrlc got shutdown signal");
        // send shutdown signal
        let shutdown_sender_c = shutdown_sender.clone();
        tokio::task::block_in_place(|| {
            rt.block_on(async {
                // trigger shutdown
                match shutdown_sender_c.send(true).await {
                    Ok(_) => {}
                    Err(_) => {}
                };
            });
        });
    })
    .expect("Error setting Ctrl-C handler");

    // print each state signal to the terminal
    let (state_signal_sender, mut state_signal_receiver) =
        tokio::sync::mpsc::channel::<StateSignal>(10);
    tokio::task::spawn(async move {
        while let Some(signal) = state_signal_receiver.recv().await {
            println!("{}", state_signal_to_stdout(&signal));
        }
    });

    let opt = Opt::from_args();

    let dna_paths = vec![
        vec!["../happs/observation/hrea_observation.dna", "hrea_observation_1"],
        vec!["../happs/planning/hrea_planning.dna", "hrea_planning_1"],
        vec!["../happs/agreement/hrea_agreement.dna", "hrea_agreement_1"],
        vec!["../happs/agent/hrea_agent.dna", "hrea_agent_1"],
        vec!["../happs/proposal/hrea_proposal.dna", "hrea_proposal_1"],
        vec!["../happs/specification/hrea_specification.dna", "hrea_specification_1"],
    ];

    // String is like "CellNick"/"SlotId"
    let dnas: Vec<(Vec<u8>, String)> = dna_paths
        .into_iter()
        .map(|path| match read(path.first().unwrap()) {
            Ok(bytes) => (bytes, path.get(1).unwrap().to_string()),
            Err(_e) => {
                println!("Failed to read dna from path: {:?}", path);
                exit(1);
            }
        })
        .collect();

    // An infinite stream of hangup signals.

    // Get a handle from this runtime
    tokio::task::block_in_place(|| {
        rt_handle.block_on(async {
            let shutdown_oneshot_sender = async_main(HcConfig {
                app_id: opt.app_id,
                dnas,
                admin_ws_port: opt.admin_ws_port,
                app_ws_port: opt.app_ws_port,
                datastore_path: opt.datastore_path,
                keystore_path: opt.keystore_path,
                proxy_url: opt.proxy_url,
                membrane_proof: opt.membrane_proof,
                event_channel: Some(state_signal_sender),
            })
            .await;
            // wait for shutdown signal
            shutdown_receiver.recv().await;
            // pass that signal through to embedded-holochain-runner
            match shutdown_oneshot_sender.send(true) {
                Ok(()) => {
                    println!("successfully sent shutdown signal to embedded-holochain-runner");
                }
                Err(_) => {
                    println!("the receiver of the oneshot sender must have been dropped");
                    panic!()
                }
            };
        })
    });
}

fn state_signal_to_stdout(signal: &StateSignal) -> i16 {
    match signal {
        StateSignal::IsFirstRun => 0,
        StateSignal::IsNotFirstRun => 1,
        // IsFirstRun events
        StateSignal::CreatingKeys => 2,
        StateSignal::RegisteringDna => 3,
        StateSignal::InstallingApp => 4,
        StateSignal::EnablingApp => 5,
        StateSignal::AddingAppInterface => 6,
        // Done/Ready Event
        StateSignal::IsReady => 7,
    }
}
