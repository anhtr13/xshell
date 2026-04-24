use std::{fmt::Display, time};

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

#[derive(Debug, Clone)]
pub struct Job {
    pub pid: u32,    // process id
    pub number: u32, // number in the job queue
    pub command: String,
    pub status: JobStatus,
    pub created_at: time::Instant,
}

pub fn recent_jobs_ids(jobs: &[Job]) -> (u32, u32) {
    let (mut a, mut b) = (&jobs[0], &jobs[0]);
    for job in jobs.iter().skip(1) {
        if job.created_at > a.created_at {
            b = a;
            a = job;
        }
    }
    (a.pid, b.pid)
}
