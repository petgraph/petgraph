//! [`Harness`] for discovering test inputs and asserting against snapshot files
//!
//! Taken from [snapbox](https://docs.rs/snapshot) and adapted to make matrix tests possible.

use std::path::Path;

use hashbrown::{HashMap, HashSet};
use ignore::{
    overrides::{Override, OverrideBuilder},
    WalkBuilder,
};
use libtest_mimic::Trial;
use snapbox::{report::Palette, Action};

pub(crate) struct Harness<S, T> {
    root: std::path::PathBuf,
    overrides: Option<Override>,
    each: Option<&'static [&'static str]>,
    setup: S,
    test: T,
    action: Action,
}

impl<S, T, I, E> Harness<S, T>
where
    I: std::fmt::Display,
    E: std::fmt::Display,
    S: Fn(std::path::PathBuf) -> Case + Send + Sync + 'static,
    T: Fn(&Path, &'static str) -> Result<I, E> + Send + Sync + 'static + Clone,
{
    pub(crate) fn new(root: impl Into<std::path::PathBuf>, setup: S, test: T) -> Self {
        Self {
            root: root.into(),
            overrides: None,
            setup,
            // in theory we would want to do this via a type-state instead,
            // but I can't be bothered to put in the extra effort, unless we upstream this.
            each: None,
            test,
            action: Action::Verify,
        }
    }

    /// Path patterns for selecting input files
    ///
    /// This used gitignore syntax
    pub(crate) fn select<'p>(mut self, patterns: impl IntoIterator<Item = &'p str>) -> Self {
        let mut overrides = OverrideBuilder::new(&self.root);
        for line in patterns {
            overrides.add(line).unwrap();
        }
        self.overrides = Some(overrides.build().unwrap());
        self
    }

    pub(crate) fn each(mut self, names: &'static [&'static str]) -> Self {
        self.each = Some(names);
        self
    }

    /// Override the failure action
    pub(crate) fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    fn trials(&self, name: &'static str) -> impl IntoIterator<Item = Trial> + '_ {
        let mut walk = WalkBuilder::new(&self.root);
        walk.standard_filters(false);
        let tests = walk.build().filter_map(|entry| {
            let entry = entry.unwrap();
            let is_dir = entry.file_type().map(|f| f.is_dir()).unwrap_or(false);
            let path = entry.into_path();
            if let Some(overrides) = &self.overrides {
                overrides
                    .matched(&path, is_dir)
                    .is_whitelist()
                    .then_some(path)
            } else {
                Some(path)
            }
        });

        tests.into_iter().map(move |path| {
            let case = (self.setup)(path);

            let test = self.test.clone();
            let trial_name = if name.is_empty() {
                case.name.clone()
            } else {
                format!("{name}::{}", case.name)
            };
            let action = self.action;

            Trial::test(trial_name, move || {
                let actual = test(&case.fixture, name)?;
                let actual = actual.to_string();

                let verify = Verifier::new();
                verify.assert(&case.fixture, &case.expected, actual)?;
                Ok(())
            })
            .with_ignored_flag(action == Action::Ignore)
        })
    }

    /// Run tests
    pub(crate) fn test(self) -> ! {
        let each = self.each.unwrap_or(&[""]);

        let tests = each.iter().flat_map(|name| self.trials(name)).collect();

        let args = libtest_mimic::Arguments::from_args();
        libtest_mimic::run(&args, tests).exit()
    }
}

struct Solution {
    cost: u64,
    edges: usize,

    path: Vec<Connection>,
}

#[derive(Debug, Copy, Clone)]
struct Connection {
    source: usize,
    target: usize,
}

struct Graph {
    nodes: usize,
    edges_len: usize,

    source: usize,
    target: usize,

    edges: HashMap<(usize, usize), u64>,
}

macro_rules! next {
    ($iter:ident as $ty:ty) => {
        $iter
            .next()
            .expect("expected token")
            .parse::<$ty>()
            .expect(concat!("is ", stringify!($ty)))
    };
}

struct Verifier {
    palette: Palette,
}

impl Verifier {
    fn new() -> Self {
        Self {
            palette: Palette::color(),
        }
    }

    fn parse_out(value: String) -> Option<Solution> {
        let mut lines = value.lines();

        let (cost, edges) = {
            let line = lines.next().expect("at least one line");
            let mut iter = line.split_whitespace();

            let cost = next!(iter as i128);
            if cost < 0 {
                return None;
            }
            let cost = cost as u64;

            let edges = next!(iter as usize);

            (cost, edges)
        };

        let mut path = Vec::with_capacity(edges);

        for _ in 0..edges {
            let line = lines.next().expect("expected line");
            let mut iter = line.split_whitespace();

            let source = next!(iter as usize);
            let target = next!(iter as usize);

            path.push(Connection { source, target });
        }

        Some(Solution { cost, edges, path })
    }

    fn parse_in(value: String) -> Graph {
        let mut lines = value.lines();

        let (nodes, edges_len, source, target) = {
            let line = lines.next().expect("at least one line");
            let mut iter = line.split_whitespace();
            let nodes = next!(iter as usize);
            let edges_len = next!(iter as usize);
            let source = next!(iter as usize);
            let target = next!(iter as usize);

            (nodes, edges_len, source, target)
        };

        let mut edges = HashMap::with_capacity(edges_len);

        for _ in 0..edges_len {
            let line = lines.next().expect("expected line");
            let mut iter = line.split_whitespace();
            let source = next!(iter as usize);
            let target = next!(iter as usize);
            let weight = next!(iter as u64);

            edges.insert((source, target), weight);
        }

        Graph {
            nodes,
            edges_len,
            source,
            target,
            edges,
        }
    }

    fn assert(&self, graph: &Path, expected: &Path, received: String) -> snapbox::Result<()> {
        let graph = std::fs::read_to_string(graph).map_err(snapbox::Error::new)?;
        let graph = Self::parse_in(graph);

        let expected = std::fs::read_to_string(expected).map_err(snapbox::Error::new)?;
        let expected = Self::parse_out(expected);

        let received = Self::parse_out(received);

        self.verify(graph, expected, received)
    }

    /// Implementation of the algorithm used in `library-checker-problems`
    ///
    /// see:
    /// <https://github.com/yosupo06/library-checker-problems/blob/1115a1bf77a8f3d66ab95e46a693a841cd4a7098/graph/shortest_path/checker.cpp>
    fn verify(
        &self,
        graph: Graph,
        expected: Option<Solution>,
        received: Option<Solution>,
    ) -> snapbox::Result<()> {
        let (expected, received) = match (expected, received) {
            (Some(expected), Some(received)) => (expected, received),
            (None, None) => return Ok(()),
            (Some(_), None) => panic!(
                "{}: expected path, but didn't receive one",
                self.palette.error("path existence differs")
            ),
            (None, Some(_)) => panic!(
                "{}: received path, but didn't expect one",
                self.palette.error("path existence differs")
            ),
        };

        let first = expected.path.first().expect("at least one edge");
        let last = expected.path.last().expect("at least one edge");

        if first.source != graph.source {
            panic!(
                "{}: path starts at wrong node",
                self.palette.error("path differs")
            );
        }

        if last.target != graph.target {
            panic!(
                "{}: path ends at wrong node",
                self.palette.error("path differs")
            );
        }

        let mut used = HashSet::new();
        used.insert(graph.source);

        let mut total = 0;

        for index in 0..expected.edges {
            let current = expected.path[index];

            if index + 1 < expected.edges {
                let next = expected.path[index + 1];

                if current.target != next.source {
                    panic!(
                        "{}: teleporting between {}th edge and {}th edge from vertex {} to {}",
                        self.palette.error("teleportation"),
                        index,
                        index + 1,
                        current.target,
                        next.source
                    );
                }
            }

            let Some(&cost) = graph.edges.get(&(current.source, current.target)) else {
                panic!(
                    "{}: edge from {} to {} doesn't exist",
                    self.palette.error("edge existence"),
                    current.source,
                    current.target
                );
            };

            if used.contains(&current.target) {
                panic!(
                    "{}: vertex {} is used twice",
                    self.palette.error("vertex usage"),
                    current.target
                );
            }

            used.insert(current.target);
            total += cost;
        }

        if total != received.cost {
            panic!(
                "{}: total weights differ between calculated ({}) and received path ({})",
                self.palette.error("total weight differs"),
                total,
                received.cost
            )
        }

        if total > expected.cost {
            panic!(
                "{}: not the shortest path, shortest {}, submitted {}",
                self.palette.error("not shortest"),
                expected.cost,
                total
            )
        }

        if total < expected.cost {
            panic!(
                "{}: submitted solution shorter than judge's solution, submitted {}, judge {}",
                self.palette.error("shorter"),
                total,
                expected.cost
            )
        }

        Ok(())
    }
}

pub(crate) struct Case {
    pub(crate) name: String,
    pub(crate) fixture: std::path::PathBuf,
    pub(crate) expected: std::path::PathBuf,
}
