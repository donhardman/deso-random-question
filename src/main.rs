use core::time;
use std::{fs::File, io::BufReader, thread, env};
use dotenv::dotenv;
use deso::seed_hex::{self, SeedHex};
use serde_json::Value;

const QUESTION_FILE: &'static str = "questions.txt";

struct Account {
    address: String,
    key_hex: SeedHex,
}

fn main() {
    dotenv().ok();
    let mnemonic = env::var("MNEMONIC").expect("MNEMONIC is not set");
    let address = env::var("ADDRESS").expect("ADDRESS is not set");
    let account = &Account {
        address,
        key_hex: seed_hex::from_mnemonic(mnemonic).expect("Error while creating seed hex from mnemonic"),
    };
    
    loop {
        // First load questions from the file
        println!("Loading questions from file {}", QUESTION_FILE);
        let questions = read_questions(QUESTION_FILE);
        let questions = match questions {
            Ok(questions) => questions,
            Err(err) => panic!("Error while reading the questions file: {:?}", err),
        };

        // Shuffle questions and iterate until we run out of it
        let questions = shuffle_questions(questions.to_owned());
        let mut i = 0_usize;
        let q_count = questions.len();
        println!("Total questions: #{}", q_count);
        while i < q_count {
            let question = &questions[i];

            println!("Question #{}: {}", i, question);

            let result = submit_post(account, question);
            if let Ok(response) = result {
                i += 1;
                println!("Tx hash hex: {:?}", &response["TxnHashHex"]);
            } else {
                eprintln!("Error while request {:?}", result);
            }

            // now wait for a 60 seconds tefore next iteration
            thread::sleep(time::Duration::from_secs(43200));
        }
    }
}

/// Shuffle questions to iterate over it to publish it
fn shuffle_questions(questions: Vec<String>) -> Vec<String> {
    use shuffle::{shuffler::Shuffler, irs::Irs};
    use rand::rngs::mock::StepRng;
    let mut shuffled = questions.into_iter().collect::<Vec<String>>();
    let mut rng = StepRng::new(2, 13);
    let mut irs = Irs::default();

    irs.shuffle(&mut shuffled, &mut rng);
    shuffled
}

/// Read questions.txt file into vector of text string that represents question
fn read_questions(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::io::BufRead;

    let mut questions: Vec<String> = vec![];
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        questions.push(line?);
    }

    Ok(questions)
}

// Helper function to submit the post and sign it with defined mnemonic in env
fn submit_post(account: &Account, text: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let author_address = account.address.as_str();
    let text = text.replace("\"", "\\\"");
    let request_json = format!(r#"
    {{
        "BodyObj": {{
            "Body": "{text}",
            "ImageURLs": [],
            "VideoURLs": []
        }},
        "IsTutorial": false,
        "IsHidden": false,
        "MinFeeRateNanosPerKB": 1000,
        "ParentStakeID": "",
        "PostHashHexToModify": "",
        "RepostedPostHashHex": "",
        "UpdaterPublicKeyBase58Check": "{author_address}"
    }}"#);
    let result = api_call("https://node.deso.org/api/v0/submit-post", request_json.as_str())?;
    let tx_hex = result["TransactionHex"].as_str().unwrap_or("");
    let signing_result = account.key_hex.sign_transaction(tx_hex);

    // If we signed tx we can broadcast it and get post hash
    if let Ok(signed_tx) = signing_result {
        let submit_json = format!(r#"
        {{
            "TransactionHex": "{signed_tx}"
        }}
        "#);
        api_call("https://node.deso.org/api/v0/submit-transaction", submit_json.as_str())
    } else {
        Err("Error while signing transaction".into())
    }
}

// Internal function to make low level API call to deso node
fn api_call(url: &str, body: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let resp = client.post(url)
        .header("Content-type", "application/json")
        .body(body.to_string())
        .send()?
        .text()?;
    let v: Value = serde_json::from_str(resp.as_str())?;
    Ok(v)
}
