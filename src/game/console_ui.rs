use crate::input::InputEvent;
use crate::model::{EntityId, GameState};
use crate::presentation::{ConsoleCandidateView, ConsoleScrollView, ConsoleView};

#[derive(Clone)]
pub(super) struct Candidate {
    pub(super) value: String,
    pub(super) hint: String,
}

#[derive(Default)]
pub(super) struct ConsoleState {
    pub(super) output: Vec<String>,
    pub(super) input: String,
    pub(super) history: Vec<String>,
    pub(super) history_index: Option<usize>,
    pub(super) scroll: ScrollPosition,
    pub(super) completion_scroll: usize,
    pub(super) candidates: Vec<Candidate>,
    pub(super) selected: usize,
    pub(super) autocomplete: bool,
    pub(super) exit: bool,
}

#[derive(Clone, Copy, Default)]
pub(super) enum ScrollPosition {
    #[default]
    Follow,
    Offset(usize),
    Home,
}

pub(super) fn build_view(console: &ConsoleState) -> ConsoleView {
    ConsoleView {
        output: console.output.clone(),
        input: console.input.clone(),
        scroll: match console.scroll {
            ScrollPosition::Follow => ConsoleScrollView::Follow,
            ScrollPosition::Offset(offset) => ConsoleScrollView::Offset(offset),
            ScrollPosition::Home => ConsoleScrollView::Home,
        },
        completion_scroll: console.completion_scroll,
        candidates: console
            .candidates
            .iter()
            .map(|candidate| ConsoleCandidateView {
                value: candidate.value.clone(),
                hint: candidate.hint.clone(),
            })
            .collect(),
        selected: console.selected,
        autocomplete: console.autocomplete,
    }
}

pub(super) fn edit_input(console: &mut ConsoleState, key: InputEvent) {
    match key {
        InputEvent::Character(c) if !c.is_control() => {
            console.input.push(c);
            console.history_index = None;
        }
        InputEvent::Backspace => {
            console.input.pop();
            console.history_index = None;
        }
        _ => {}
    }
}

pub(super) fn history_previous(console: &mut ConsoleState) {
    if console.history.is_empty() {
        return;
    }
    let index = console
        .history_index
        .map_or(console.history.len() - 1, |i| i.saturating_sub(1));
    console.history_index = Some(index);
    console.input = console.history[index].clone();
}

pub(super) fn history_next(console: &mut ConsoleState) {
    let Some(index) = console.history_index else {
        return;
    };
    if index + 1 >= console.history.len() {
        console.history_index = None;
        console.input.clear();
        return;
    }
    let index = index + 1;
    console.history_index = Some(index);
    console.input = console.history[index].clone();
}

pub(super) fn scroll_up(console: &mut ConsoleState, amount: usize) {
    console.scroll = match console.scroll {
        ScrollPosition::Follow => ScrollPosition::Offset(amount),
        ScrollPosition::Offset(offset) => ScrollPosition::Offset(offset.saturating_add(amount)),
        ScrollPosition::Home => ScrollPosition::Home,
    };
}

pub(super) fn scroll_down(console: &mut ConsoleState, amount: usize) {
    console.scroll = match console.scroll {
        ScrollPosition::Follow => ScrollPosition::Follow,
        ScrollPosition::Home => ScrollPosition::Offset(0),
        ScrollPosition::Offset(offset) => {
            let next = offset.saturating_sub(amount);
            if next == 0 {
                ScrollPosition::Follow
            } else {
                ScrollPosition::Offset(next)
            }
        }
    };
}

pub(super) fn jump_home(console: &mut ConsoleState) {
    console.scroll = ScrollPosition::Home;
}

pub(super) fn jump_end(console: &mut ConsoleState) {
    console.scroll = ScrollPosition::Follow;
}

pub(super) fn refresh_completion(console: &mut ConsoleState, state: &GameState) {
    let (tokens, trailing) = tokenize(&console.input);
    let prefix = if trailing {
        ""
    } else {
        tokens.last().map(String::as_str).unwrap_or("")
    };
    let mut candidates = Vec::new();

    if tokens.len() <= 1 && !trailing {
        for (value, hint) in command_candidates() {
            if value.starts_with(prefix) {
                candidates.push(Candidate { value, hint });
            }
        }
    } else {
        match tokens.first().map(String::as_str) {
            Some("goto") | Some("teleport") => {
                candidates = entity_candidates(
                    state
                        .world
                        .locations
                        .iter()
                        .map(|location| (location.id, location.name.clone())),
                    prefix,
                )
            }
            Some("quest") => {
                if tokens.len() <= 2 {
                    for value in ["list", "complete", "reset"] {
                        if value.starts_with(prefix) {
                            candidates.push(Candidate {
                                value: value.into(),
                                hint: "quest subcommand".into(),
                            });
                        }
                    }
                } else if matches!(
                    tokens.get(1).map(String::as_str),
                    Some("complete" | "reset")
                ) {
                    candidates = entity_candidates(
                        state
                            .quests
                            .iter()
                            .map(|quest| (quest.id, quest.title.clone())),
                        prefix,
                    );
                }
            }
            Some("faction") => {
                if tokens.len() <= 2 && "set".starts_with(prefix) {
                    candidates.push(Candidate {
                        value: "set".into(),
                        hint: "set reputation".into(),
                    });
                } else if tokens.get(1).map(String::as_str) == Some("set") {
                    candidates = entity_candidates(
                        state
                            .factions
                            .iter()
                            .map(|faction| (faction.id, faction.name.clone())),
                        prefix,
                    );
                }
            }
            Some("npc") | Some("npcs") => {
                candidates = entity_candidates(
                    state.npcs.iter().map(|npc| (npc.id, npc.display_name())),
                    prefix,
                )
            }
            Some("give") | Some("remove") => {
                candidates = entity_candidates(
                    state
                        .character
                        .inventory
                        .iter()
                        .map(|item| (item.id, item.name.clone())),
                    prefix,
                )
            }
            _ => {}
        }
    }

    let candidates_changed = console.candidates.len() != candidates.len()
        || console
            .candidates
            .iter()
            .zip(candidates.iter())
            .any(|(left, right)| left.value != right.value || left.hint != right.hint);

    console.candidates = candidates;
    if console.candidates.is_empty() {
        console.autocomplete = false;
        console.completion_scroll = 0;
        return;
    }
    if console.selected >= console.candidates.len() {
        console.selected = 0;
    }
    if candidates_changed {
        console.completion_scroll = console
            .completion_scroll
            .min(console.candidates.len().saturating_sub(1));
    }
    keep_completion_selection_visible(console, 8);
}

fn entity_candidates<I>(entities: I, prefix: &str) -> Vec<Candidate>
where
    I: IntoIterator<Item = (EntityId, String)>,
{
    entities
        .into_iter()
        .filter_map(|(id, hint)| {
            let value = id.to_string();
            value
                .starts_with(prefix)
                .then_some(Candidate { value, hint })
        })
        .collect()
}

fn tokenize(input: &str) -> (Vec<String>, bool) {
    (
        input.split_whitespace().map(ToOwned::to_owned).collect(),
        input.chars().last().is_some_and(char::is_whitespace),
    )
}

fn command_candidates() -> Vec<(String, String)> {
    [
        ("help", "show commands"),
        ("clear", "clear output"),
        ("status", "character/world state"),
        ("where", "filesystem content paths"),
        ("mods", "external mod directories"),
        ("content", "loaded content diagnostics"),
        ("locations", "list location IDs"),
        ("goto", "move to location ID"),
        ("teleport", "goto alias"),
        ("npcs", "list NPC IDs"),
        ("quests", "list quests"),
        ("quest", "quest commands"),
        ("factions", "list factions"),
        ("faction", "faction commands"),
        ("inventory", "list inventory"),
        ("give", "clone an item"),
        ("remove", "remove an item"),
        ("heal", "restore HP"),
        ("damage", "deal damage"),
        ("kill", "set alive=false"),
        ("revive", "set alive=true"),
        ("xp", "set/add experience"),
        ("level", "set level"),
        ("attr", "set attribute"),
        ("condition", "add/clear condition"),
        ("time", "set/add time points"),
        ("day", "set day"),
        ("history", "recent history"),
        ("reload", "reload campaign content"),
        ("save", "save current game"),
        ("exit", "close console"),
    ]
    .into_iter()
    .map(|(value, hint)| (value.into(), hint.into()))
    .collect()
}

pub(super) fn keep_completion_selection_visible(console: &mut ConsoleState, visible: usize) {
    if console.candidates.is_empty() || visible == 0 {
        console.completion_scroll = 0;
        return;
    }
    let max_start = console.candidates.len().saturating_sub(visible);
    if console.selected < console.completion_scroll {
        console.completion_scroll = console.selected;
    } else if console.selected >= console.completion_scroll + visible {
        console.completion_scroll = console.selected + 1 - visible;
    }
    console.completion_scroll = console.completion_scroll.min(max_start);
}

pub(super) fn select_previous(console: &mut ConsoleState) {
    if !console.candidates.is_empty() {
        console.selected = console
            .selected
            .checked_sub(1)
            .unwrap_or(console.candidates.len() - 1);
        keep_completion_selection_visible(console, 8);
    }
}

pub(super) fn select_next(console: &mut ConsoleState) {
    if !console.candidates.is_empty() {
        console.selected = (console.selected + 1) % console.candidates.len();
        keep_completion_selection_visible(console, 8);
    }
}

pub(super) fn cancel_completion(console: &mut ConsoleState) {
    console.autocomplete = false;
    console.candidates.clear();
    console.completion_scroll = 0;
}

pub(super) fn accept_completion(console: &mut ConsoleState) {
    let Some(candidate) = console.candidates.get(console.selected).cloned() else {
        return;
    };
    let (mut tokens, trailing) = tokenize(&console.input);
    if trailing {
        tokens.push(candidate.value);
    } else if let Some(last) = tokens.last_mut() {
        *last = candidate.value;
    } else {
        tokens.push(candidate.value);
    }
    console.input = tokens.join(" ");
    cancel_completion(console);
}
