use std::{collections::HashSet, fmt::Display, process::Child};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JobStatus {
    Running,
    Done,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Done => write!(f, "Done"),
        }
    }
}

#[derive(Debug)]
pub struct Job {
    pub child: Child,
    pub number: u32, // number in the job queue
    pub command: String,
    pub status: JobStatus,
}

pub struct Jobs {
    jobs: Vec<Job>,
    number_pool: HashSet<u32>,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            jobs: Vec::new(),
            number_pool: HashSet::new(),
        }
    }

    pub fn push(&mut self, job: Job) {
        self.number_pool.insert(job.number);
        self.jobs.push(job);
    }

    pub fn new_job_number(&self) -> u32 {
        let mut num = 1;
        while self.number_pool.contains(&num) {
            num += 1;
        }
        num
    }

    pub fn print_done(&mut self) {
        for (i, job) in self.jobs.iter().enumerate() {
            let marker = if i + 1 == self.jobs.len() {
                "+"
            } else if i + 2 == self.jobs.len() {
                "-"
            } else {
                " "
            };
            if job.status == JobStatus::Done {
                println!(
                    "[{}]{}  Done                    {}",
                    job.number, marker, job.command
                );
            }
        }
    }

    pub fn update_status(&mut self) {
        for job in self.jobs.iter_mut() {
            match job.child.try_wait() {
                Ok(status) => {
                    if status.is_some() {
                        job.status = JobStatus::Done;
                    }
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    job.status = JobStatus::Done;
                }
            }
        }
    }

    pub fn clean_up(&mut self) {
        for job in self.jobs.iter() {
            if job.status == JobStatus::Done {
                self.number_pool.remove(&job.number);
            }
        }
        self.jobs.retain(|job| job.status == JobStatus::Running);
    }

    pub fn value(&self) -> &[Job] {
        &self.jobs
    }
}
