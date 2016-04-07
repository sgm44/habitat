// Copyright:: Copyright (c) 2015-2016 Chef Software, Inc.
//
// The terms of the Evaluation Agreement (Habitat) between Chef Software Inc.
// and the party accessing this file ("Licensee") apply to Licensee's use of
// the Software until such time that the Software is made available under an
// open source license such as the Apache 2.0 License.

#[macro_use]
extern crate habitat_sup as sup;
extern crate habitat_core as hcore;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ansi_term;
extern crate libc;
#[macro_use]
extern crate clap;

use std::ffi::CString;
use std::process;
use std::ptr;
use std::result;
use std::str::FromStr;

use ansi_term::Colour::Yellow;
use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};
use hcore::package::PackageIdent;
use hcore::fs;

use sup::config::{Command, Config, UpdateStrategy};
use sup::error::{Error, Result, SupError};
use sup::command::*;
use sup::topology::Topology;

/// Our output key
static LOGKEY: &'static str = "MN";

/// The version number
#[allow(dead_code)]
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// CLI defaults
static DEFAULT_GROUP: &'static str = "default";
static DEFAULT_GOSSIP_LISTEN: &'static str = "0.0.0.0:9634";
static DEFAULT_DEPOT_URL: &'static str = "http://52.37.151.35:9632";

/// Creates a [Config](config/struct.Config.html) from global args
/// and subcommand args.
fn config_from_args(args: &ArgMatches, subcommand: &str, sub_args: &ArgMatches) -> Result<Config> {
    let mut config = Config::new();
    let command = try!(Command::from_str(subcommand));
    config.set_command(command);
    if let Some(ref strategy) = sub_args.value_of("strategy") {
        config.set_update_strategy(UpdateStrategy::from_str(strategy));
    }
    if let Some(ref archive) = sub_args.value_of("archive") {
        config.set_archive(archive.to_string());
    }
    if let Some(ref package) = sub_args.value_of("package") {
        let ident = try!(PackageIdent::from_str(package));
        config.set_package(ident);
    }
    if let Some(key) = sub_args.value_of("key") {
        config.set_key(key.to_string());
    }
    if let Some(password) = sub_args.value_of("password") {
        config.set_password(password.to_string());
    }
    if let Some(email) = sub_args.value_of("email") {
        config.set_email(email.to_string());
    }
    if let Some(user) = sub_args.value_of("user") {
        config.set_user_key(user.to_string());
    }
    if let Some(service) = sub_args.value_of("service") {
        config.set_service_key(service.to_string());
    }
    if let Some(infile) = sub_args.value_of("infile") {
        config.set_infile(infile.to_string());
    }
    if let Some(outfile) = sub_args.value_of("outfile") {
        config.set_outfile(outfile.to_string());
    }
    if let Some(topology) = sub_args.value_of("topology") {
        match topology.as_ref() {
            "standalone" => {
                config.set_topology(Topology::Standalone);
            }
            "leader" => {
                config.set_topology(Topology::Leader);
            }
            "initializer" => {
                config.set_topology(Topology::Initializer);
            }
            t => return Err(sup_error!(Error::UnknownTopology(String::from(t)))),
        }
    }
    if sub_args.value_of("expire-days").is_some() {
        let ed = value_t!(sub_args.value_of("expire-days"), u16).unwrap_or_else(|e| e.exit());
        config.set_expire_days(ed);
    }
    config.set_url(sub_args.value_of("url").unwrap_or(DEFAULT_DEPOT_URL).to_string());
    config.set_group(sub_args.value_of("group").unwrap_or(DEFAULT_GROUP).to_string());
    let watches = match sub_args.values_of("watch") {
        Some(ws) => ws.map(|s| s.to_string()).collect(),
        None => vec![],
    };
    config.set_watch(watches);
    config.set_path(sub_args.value_of("path")
                            .unwrap_or(fs::service_path(sup::PROGRAM_NAME.as_str())
                                           .join("data")
                                           .to_string_lossy()
                                           .as_ref())
                            .to_string());
    config.set_gossip_listen(sub_args.value_of("gossip-listen")
                                     .unwrap_or(DEFAULT_GOSSIP_LISTEN)
                                     .to_string());
    let gossip_peers = match sub_args.values_of("gossip-peer") {
        Some(gp) => gp.map(|s| s.to_string()).collect(),
        None => vec![],
    };
    config.set_gossip_peer(gossip_peers);
    if sub_args.value_of("gossip-permanent").is_some() {
        config.set_gossip_permanent(true);
    }
    if let Some(sg) = sub_args.value_of("service-group") {
        config.set_service_group(sg.to_string());
    }
    if let Some(fp) = sub_args.value_of("file-path") {
        config.set_file_path(fp.to_string());
    }
    config.set_version_number(value_t!(sub_args, "version-number", u64).unwrap_or(0));
    if args.value_of("verbose").is_some() {
        sup::output::set_verbose(true);
    }
    if args.value_of("no-color").is_some() {
        sup::output::set_no_color(true);
    }
    debug!("Config:\n{:?}", config);
    Ok(config)
}

type Handler = fn(&Config) -> result::Result<(), sup::error::SupError>;

/// The entrypoint for the Supervisor.
///
/// * Set up the logger
/// * Pull in the arguments from the Command Line, push through clap
/// * Dispatch to a function that handles that action called
/// * Exit cleanly, or if we return an `Error`, call `exit_with(E, 1)`
#[allow(dead_code)]
fn main() {
    env_logger::init().unwrap();

    let arg_url = || {
        Arg::with_name("url")
            .short("u")
            .long("url")
            .takes_value(true)
            .help("Use the specified package depot url")
    };
    let arg_group = || {
        Arg::with_name("group")
            .long("group")
            .takes_value(true)
            .help("The service group; shared config and topology [default: default].")
    };
    let arg_infile = || {
        Arg::with_name("infile")
            .long("infile")
            .takes_value(true)
            .help("Input filename")
    };
    let arg_outfile = || {
        Arg::with_name("outfile")
            .long("outfile")
            .takes_value(true)
            .help("Output filename")
    };
    let arg_strategy = || {
        Arg::with_name("strategy")
            .long("strategy")
            .short("s")
            .takes_value(true)
            .possible_values(&["none", "at-once"])
            .help("The update strategy; [default: none].")
    };

    let sub_start = SubCommand::with_name("start")
                        .about("Start a Habitat-supervised service from a package")
                        .arg(Arg::with_name("package")
                                 .index(1)
                                 .required(true)
                                 .help("Name of package to start"))
                        .arg(arg_url())
                        .arg(arg_group())
                        .arg(arg_strategy())
                        .arg(Arg::with_name("topology")
                                 .short("t")
                                 .long("topology")
                                 .value_name("topology")
                                 .help("Service topology"))
                        .arg(Arg::with_name("watch")
                                 .long("watch")
                                 .value_name("watch")
                                 .multiple(true)
                                 .help("One or more service groups to watch for updates"))
                        .arg(Arg::with_name("gossip-peer")
                                 .long("gossip-peer")
                                 .value_name("ip:port")
                                 .multiple(true)
                                 .help("The listen string of an initial gossip peer"))
                        .arg(Arg::with_name("gossip-listen")
                                 .long("gossip-listen")
                                 .value_name("ip:port")
                                 .help("The listen string for gossip [default: 0.0.0.0:9634]"))
                        .arg(Arg::with_name("gossip-permanant")
                                 .short("I")
                                 .long("gossip-permanent")
                                 .help("If this service is a permanent gossip peer"));
    let sub_sh = SubCommand::with_name("sh").about("Start an interactive shell");
    let sub_generate_user_key = SubCommand::with_name("generate-user-key")
                                    .about("Generate a Habitat user key")
                                    .arg(Arg::with_name("user")
                                             .required(true)
                                             .long("user")
                                             .takes_value(true)
                                             .help("Name of user key"))
                                    .arg(Arg::with_name("password")
                                             .required(false)
                                             .long("password")
                                             .takes_value(true)
                                             .help("User key password"))
                                    .arg(Arg::with_name("email")
                                             .required(true)
                                             .long("email")
                                             .takes_value(true)
                                             .help("User key email address"))
                                    .arg(Arg::with_name("expire-days")
                                             .long("expire-days")
                                             .takes_value(true)
                                             .value_name("expire-days")
                                             .help("Number of days before a key expires"));
    let sub_generate_service_key = SubCommand::with_name("generate-service-key")
                                       .about("Generate a Habitat service key")
                                       .arg(Arg::with_name("service")
                                                .required(true)
                                                .takes_value(true)
                                                .help("Name of service key"))
                                       .arg(arg_group())
                                       .arg(Arg::with_name("expire-days")
                                                .long("expire-days")
                                                .takes_value(true)
                                                .value_name("expire-days")
                                                .help("Number of days before a key expires"));
    let sub_encrypt = SubCommand::with_name("encrypt")
                          .about("Encrypt and sign a message with a service as the recipient")
                          .arg(Arg::with_name("user")
                                   .required(true)
                                   .long("user")
                                   .takes_value(true)
                                   .help("Name of user key"))
                          .arg(Arg::with_name("service")
                                   .required(true)
                                   .long("service")
                                   .takes_value(true)
                                   .help("Name of service key"))
                          .arg(arg_infile().required(true))
                          .arg(arg_outfile().required(true))
                          .arg(Arg::with_name("password")
                                   .required(false)
                                   .long("password")
                                   .takes_value(true)
                                   .help("User key password"))
                          .arg(arg_group());
    let sub_decrypt = SubCommand::with_name("decrypt")
                          .about("Decrypt and verify a message")
                          .arg(arg_infile().required(true))
                          .arg(arg_outfile().required(true));
    let sub_import_key = SubCommand::with_name("import-key")
                             .about("Import a public Habitat key")
                             .arg(arg_infile())
                             .arg(Arg::with_name("key")
                                      .long("key")
                                      .takes_value(true)
                                      .help("Public key filename")
                                      .requires("url"))
                             .arg(arg_url())
                             .group(ArgGroup::with_name("input-method")
                                        .required(true)
                                        .args(&["infile", "key"]));
    let sub_export_key = SubCommand::with_name("export-key")
                             .about("Export a public Habitat key")
                             .arg(Arg::with_name("user")
                                      .long("user")
                                      .takes_value(true)
                                      .help("Name of user key"))
                             .arg(Arg::with_name("service")
                                      .long("service")
                                      .takes_value(true)
                                      .help("Name of service key"))
                             .group(ArgGroup::with_name("user-or-service-key")
                                        .required(true)
                                        .args(&["user", "service"]))
                             .arg(arg_outfile().required(true))
                             .arg(arg_group());
    let sub_list_keys = SubCommand::with_name("list-keys").about("List user and service keys");
    let sub_config = SubCommand::with_name("config")
                         .about("Print the default.toml for a given package")
                         .arg(Arg::with_name("package")
                                  .index(1)
                                  .required(true)
                                  .help("Name of package"));
    let args = App::new(sup::PROGRAM_NAME.as_str())
                   .version(VERSION)
                   .setting(AppSettings::VersionlessSubcommands)
                   .setting(AppSettings::SubcommandRequiredElseHelp)
                   .arg(Arg::with_name("verbose")
                            .short("v")
                            .global(true)
                            .help("Verbose output; shows line numbers"))
                   .arg(Arg::with_name("no-color")
                            .long("no-color")
                            .global(true)
                            .help("Turn ANSI color off :("))
                   .subcommand(sub_start)
                   .subcommand(sub_sh)
                   .subcommand(sub_generate_user_key)
                   .subcommand(sub_generate_service_key)
                   .subcommand(sub_encrypt)
                   .subcommand(sub_decrypt)
                   .subcommand(sub_import_key)
                   .subcommand(sub_export_key)
                   .subcommand(sub_list_keys)
                   .subcommand(sub_config);
    let matches = args.get_matches();

    debug!("clap matches {:?}", matches);

    let subcommand_name = matches.subcommand_name().unwrap();
    let subcommand_matches = matches.subcommand_matches(subcommand_name).unwrap();

    let config = match config_from_args(&matches, subcommand_name, &subcommand_matches) {
        Ok(config) => config,
        Err(e) => return exit_with(e, 1),
    };

    let result = match config.command() {
        Command::Shell => shell(&config),
        Command::Config => configure(&config),
        Command::Decrypt => decrypt(&config),
        Command::Encrypt => encrypt(&config),
        Command::ExportKey => export_key(&config),
        Command::GenerateServiceKey => generate_service_key(&config),
        Command::GenerateUserKey => generate_user_key(&config),
        Command::ImportKey => import_key(&config),
        Command::ListKeys => list_keys(&config),
        Command::Start => start(&config),
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => exit_with(e, 1),
    }
}

/// Exit with an error message and the right status code
#[allow(dead_code)]
fn exit_with(e: SupError, code: i32) {
    println!("{}", e);
    process::exit(code)
}

/// Start a shell
#[allow(dead_code)]
fn shell(_config: &Config) -> Result<()> {
    outputln!("Starting your shell; enjoy!");
    let shell_arg = try!(CString::new("sh"));
    let mut argv = [shell_arg.as_ptr(), ptr::null()];
    // Yeah, you don't know any better.. but we aren't coming back from
    // what happens next.
    unsafe {
        libc::execvp(shell_arg.as_ptr(), argv.as_mut_ptr());
    }
    Ok(())
}

/// Show the configuration options for a service
#[allow(dead_code)]
fn configure(config: &Config) -> Result<()> {
    try!(configure::display(config));
    Ok(())
}

/// Start a service
#[allow(dead_code)]
fn start(config: &Config) -> Result<()> {
    outputln!("Starting {}",
              Yellow.bold().paint(config.package().to_string()));
    try!(start::package(config));
    outputln!("Finished with {}",
              Yellow.bold().paint(config.package().to_string()));
    Ok(())
}

/// Import a key
fn import_key(config: &Config) -> Result<()> {
    outputln!("Importing key {}", Yellow.bold().paint(config.key()));
    try!(key::import(&config));
    outputln!("Finished importing key");
    Ok(())
}

/// Export a key
fn export_key(config: &Config) -> Result<()> {
    outputln!("Exporting key {}", Yellow.bold().paint(config.key()));
    try!(key::export(&config));
    outputln!("Finished exporting key");
    Ok(())
}

/// Generate a key for a user
fn generate_user_key(config: &Config) -> Result<()> {
    outputln!("Generate user key for {}",
              Yellow.bold().paint(config.key()));
    try!(key::generate_user_key(&config));
    outputln!("Finished generating user key for {}",
              Yellow.bold().paint(config.key()));
    Ok(())
}

/// Generate a key for a service
fn generate_service_key(config: &Config) -> Result<()> {
    outputln!("Generate service key for {}",
              Yellow.bold().paint(config.key()));
    try!(key::generate_service_key(&config));
    outputln!("Finished generating service key for {}",
              Yellow.bold().paint(config.key()));
    Ok(())
}

/// List managed gpg keys
fn list_keys(config: &Config) -> Result<()> {
    outputln!("Listing keys");
    try!(key::list(&config));
    outputln!("Finished listing keys");
    Ok(())
}

/// Encrypt a file
fn encrypt(config: &Config) -> Result<()> {
    outputln!("Encrypting");
    try!(key::encrypt_and_sign(&config));
    outputln!("Finished encrypting");
    Ok(())
}

/// Decrypt a file
fn decrypt(config: &Config) -> Result<()> {
    outputln!("Decrypting");
    try!(key::decrypt_and_verify(&config));
    outputln!("Finished decrypting");
    Ok(())
}
