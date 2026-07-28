use crate::builtin::Output;

/// Invariant: no `Done` job survives between public calls — every method that
/// marks jobs Done (`list`, `reap`) also removes them before returning. This
/// is what lets `reap` announce every Done job it finds as freshly exited,
/// with no "already announced" bookkeeping.
pub struct Jobs {
    jobs: Vec<Job>,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs { jobs: Vec::new() }
    }

    pub fn add(&mut self, child: std::process::Child, command: String) -> u32 {
        let id = self.get_next_id();
        let job = Job {
            id,
            child,
            command,
            status: JobStatus::Running,
        };
        self.jobs.push(job);
        id
    }

    pub fn list(&mut self, output: &mut Output) {
        self.refresh();
        for (i, job) in self.jobs.iter().enumerate() {
            let marker = self.get_marker(i);
            let job_line = self.format_line(job, marker);

            writeln!(output.stdout, "{job_line}").expect("Error writing to stdout");
        }
        self.remove_done();
    }

    pub fn reap(&mut self) {
        self.refresh();
        for (i, job) in self.jobs.iter().enumerate() {
            match job.status {
                JobStatus::Done => {
                    let marker = self.get_marker(i);
                    let job_line = self.format_line(job, marker);
                    println!("{job_line}");
                }
                JobStatus::Running => (),
            }
        }
        self.remove_done();
    }

    fn get_next_id(&self) -> u32 {
        let mut id = 1;
        loop {
            let available = self.jobs.iter().all(|job| job.id != id);
            if available {
                break;
            }
            id += 1;
        }
        id
    }

    fn refresh(&mut self) {
        self.jobs.iter_mut().for_each(|job| {
            if let Ok(Some(_)) = job.child.try_wait() {
                job.status = JobStatus::Done;
            }
        });
    }

    fn get_marker(&self, job_index: usize) -> &'static str {
        match job_index {
            i if i + 1 == self.jobs.len() => "+",
            i if i + 2 == self.jobs.len() => "-",
            _ => " ",
        }
    }

    fn format_line(&self, job: &Job, marker: &str) -> String {
        format!(
            "[{}]{marker}  {:>24} {}",
            job.id,
            job.status.as_str(),
            job.command
        )
    }

    fn remove_done(&mut self) {
        self.jobs.retain(|job| job.status != JobStatus::Done);
    }
}

struct Job {
    id: u32,
    child: std::process::Child,
    command: String,
    status: JobStatus,
}

#[derive(PartialEq, Eq)]
enum JobStatus {
    Running,
    Done,
}

impl JobStatus {
    fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Running => "Running",
            JobStatus::Done => "Done",
        }
    }
}
