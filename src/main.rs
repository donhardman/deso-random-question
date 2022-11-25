use core::time;
use std::{fs::File, io::BufReader, thread};

const QUESTION_FILE: &'static str = "questions.txt";
fn main() {
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
            i += 1;

            // now wait for a 60 seconds tefore next iteration
            thread::sleep(time::Duration::from_secs(60));
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
