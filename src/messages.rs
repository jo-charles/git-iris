use crate::ui::{
    AURORA_GREEN, CELESTIAL_BLUE, COMET_ORANGE, GALAXY_PINK, METEOR_RED, NEBULA_PURPLE,
    PLASMA_CYAN, SOLAR_YELLOW, STARLIGHT,
};
use rand::prelude::*;
use ratatui::style::Color;
use std::sync::LazyLock;

#[derive(Clone)]
pub struct ColoredMessage {
    pub text: String,
    pub color: Color,
}

static WAITING_MESSAGES: LazyLock<Vec<ColoredMessage>> = LazyLock::new(|| {
    vec![
        // Cosmic vibes
        ColoredMessage {
            text: "🔮 Consulting the commit oracle...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "✨ Weaving stardust into your message...".to_string(),
            color: STARLIGHT,
        },
        ColoredMessage {
            text: "🌌 Exploring the commit-verse...".to_string(),
            color: CELESTIAL_BLUE,
        },
        ColoredMessage {
            text: "🔭 Peering through the code telescope...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "⭐ Aligning the celestial diffs...".to_string(),
            color: STARLIGHT,
        },
        ColoredMessage {
            text: "🌙 Reading your changes by moonlight...".to_string(),
            color: CELESTIAL_BLUE,
        },
        // Nerdy & clever
        ColoredMessage {
            text: "🎲 Rolling for commit inspiration...".to_string(),
            color: AURORA_GREEN,
        },
        ColoredMessage {
            text: "🧬 Decoding the DNA of your changes...".to_string(),
            color: GALAXY_PINK,
        },
        ColoredMessage {
            text: "🔬 Analyzing diff particles...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "📡 Tuning into the commit frequency...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "🧪 Distilling the essence of your changes...".to_string(),
            color: AURORA_GREEN,
        },
        ColoredMessage {
            text: "⚡ Parsing the diff matrix...".to_string(),
            color: SOLAR_YELLOW,
        },
        // Warm & grounded
        ColoredMessage {
            text: "☕ Brewing a fresh commit message...".to_string(),
            color: COMET_ORANGE,
        },
        ColoredMessage {
            text: "🎨 Painting your changes in prose...".to_string(),
            color: GALAXY_PINK,
        },
        ColoredMessage {
            text: "🧩 Piecing together the story...".to_string(),
            color: CELESTIAL_BLUE,
        },
        ColoredMessage {
            text: "🎵 Composing a commit symphony...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "💎 Polishing your commit to a shine...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "🌿 Growing ideas from your diff...".to_string(),
            color: AURORA_GREEN,
        },
        // Playful
        ColoredMessage {
            text: "🚀 Launching into commit space...".to_string(),
            color: METEOR_RED,
        },
        ColoredMessage {
            text: "🗺️ Charting the diff territory...".to_string(),
            color: SOLAR_YELLOW,
        },
        ColoredMessage {
            text: "🌊 Riding the code waves...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "🦉 Consulting the git guardians...".to_string(),
            color: CELESTIAL_BLUE,
        },
        ColoredMessage {
            text: "🧭 Calibrating the commit compass...".to_string(),
            color: COMET_ORANGE,
        },
        ColoredMessage {
            text: "🔐 Unlocking the secrets of your diff...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "🎁 Wrapping up your changes nicely...".to_string(),
            color: STARLIGHT,
        },
        ColoredMessage {
            text: "🏄 Surfing the staged changes...".to_string(),
            color: AURORA_GREEN,
        },
    ]
});

static REVIEW_WAITING_MESSAGES: LazyLock<Vec<ColoredMessage>> = LazyLock::new(|| {
    vec![
        // Cosmic & mystical
        ColoredMessage {
            text: "🔮 Gazing into the code quality crystal...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "✨ Illuminating the hidden corners...".to_string(),
            color: STARLIGHT,
        },
        ColoredMessage {
            text: "🌟 Channeling review wisdom...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "🌙 Meditating on your abstractions...".to_string(),
            color: CELESTIAL_BLUE,
        },
        ColoredMessage {
            text: "🔭 Scanning the code horizon...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "⭐ Reading the code constellations...".to_string(),
            color: STARLIGHT,
        },
        // Nerdy & technical
        ColoredMessage {
            text: "🔬 Analyzing code under the microscope...".to_string(),
            color: AURORA_GREEN,
        },
        ColoredMessage {
            text: "🧬 Sequencing your code genome...".to_string(),
            color: GALAXY_PINK,
        },
        ColoredMessage {
            text: "📡 Scanning for code anomalies...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "🧪 Running quality experiments...".to_string(),
            color: AURORA_GREEN,
        },
        ColoredMessage {
            text: "⚡ Tracing the logic pathways...".to_string(),
            color: SOLAR_YELLOW,
        },
        ColoredMessage {
            text: "🎲 Rolling perception checks...".to_string(),
            color: COMET_ORANGE,
        },
        // Exploratory
        ColoredMessage {
            text: "🗺️ Mapping your code architecture...".to_string(),
            color: SOLAR_YELLOW,
        },
        ColoredMessage {
            text: "🔍 Hunting for hidden issues...".to_string(),
            color: METEOR_RED,
        },
        ColoredMessage {
            text: "🧭 Navigating your control flow...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "🏊 Diving into the logic depths...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "⛏️ Mining for code gems...".to_string(),
            color: COMET_ORANGE,
        },
        ColoredMessage {
            text: "🌊 Flowing through your functions...".to_string(),
            color: CELESTIAL_BLUE,
        },
        // Warm & grounded
        ColoredMessage {
            text: "☕ Taking a thoughtful look...".to_string(),
            color: COMET_ORANGE,
        },
        ColoredMessage {
            text: "🎨 Appreciating your code craft...".to_string(),
            color: GALAXY_PINK,
        },
        ColoredMessage {
            text: "🧩 Piecing together the full picture...".to_string(),
            color: CELESTIAL_BLUE,
        },
        ColoredMessage {
            text: "💎 Searching for rough edges to polish...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "🦉 Consulting the wise owl...".to_string(),
            color: AURORA_GREEN,
        },
        ColoredMessage {
            text: "📜 Checking against best practices...".to_string(),
            color: SOLAR_YELLOW,
        },
        ColoredMessage {
            text: "🎵 Listening to your code's rhythm...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "🌿 Tending the code garden...".to_string(),
            color: AURORA_GREEN,
        },
    ]
});

static USER_MESSAGES: LazyLock<Vec<ColoredMessage>> = LazyLock::new(|| {
    vec![
        ColoredMessage {
            text: "🚀 Launching...".to_string(),
            color: METEOR_RED,
        },
        ColoredMessage {
            text: "✨ Working magic...".to_string(),
            color: STARLIGHT,
        },
        ColoredMessage {
            text: "🔮 Divining...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "⚡ Processing...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "🌌 Exploring...".to_string(),
            color: CELESTIAL_BLUE,
        },
        ColoredMessage {
            text: "🔬 Analyzing...".to_string(),
            color: AURORA_GREEN,
        },
        ColoredMessage {
            text: "☕ Brewing...".to_string(),
            color: COMET_ORANGE,
        },
        ColoredMessage {
            text: "🎨 Crafting...".to_string(),
            color: GALAXY_PINK,
        },
        ColoredMessage {
            text: "🧩 Piecing...".to_string(),
            color: CELESTIAL_BLUE,
        },
        ColoredMessage {
            text: "💎 Polishing...".to_string(),
            color: PLASMA_CYAN,
        },
        ColoredMessage {
            text: "🎵 Composing...".to_string(),
            color: NEBULA_PURPLE,
        },
        ColoredMessage {
            text: "🌊 Flowing...".to_string(),
            color: AURORA_GREEN,
        },
        ColoredMessage {
            text: "🔭 Scanning...".to_string(),
            color: SOLAR_YELLOW,
        },
        ColoredMessage {
            text: "🧪 Testing...".to_string(),
            color: COMET_ORANGE,
        },
        ColoredMessage {
            text: "🌿 Growing...".to_string(),
            color: AURORA_GREEN,
        },
    ]
});

pub fn get_waiting_message() -> ColoredMessage {
    let mut rng = rand::rng();
    WAITING_MESSAGES
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| ColoredMessage {
            text: "Processing your request...".to_string(),
            color: SOLAR_YELLOW,
        })
}

pub fn get_review_waiting_message() -> ColoredMessage {
    let mut rng = rand::rng();
    REVIEW_WAITING_MESSAGES
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| ColoredMessage {
            text: "Analyzing your code quality...".to_string(),
            color: NEBULA_PURPLE,
        })
}

/// Get a waiting message appropriate for the given capability
pub fn get_capability_message(capability: &str) -> ColoredMessage {
    match capability {
        "review" => get_review_waiting_message(),
        "pr" => get_pr_waiting_message(),
        "changelog" => get_changelog_waiting_message(),
        "release_notes" => get_release_notes_waiting_message(),
        // "commit" and any other capability use the default cosmic messages
        _ => get_waiting_message(),
    }
}

static PR_WAITING_MESSAGES: std::sync::LazyLock<Vec<ColoredMessage>> =
    std::sync::LazyLock::new(|| {
        vec![
            ColoredMessage {
                text: "🔮 Crafting your PR narrative...".to_string(),
                color: NEBULA_PURPLE,
            },
            ColoredMessage {
                text: "✨ Weaving your commits into a story...".to_string(),
                color: STARLIGHT,
            },
            ColoredMessage {
                text: "📝 Summarizing your brilliant work...".to_string(),
                color: CELESTIAL_BLUE,
            },
            ColoredMessage {
                text: "🎯 Distilling the essence of your changes...".to_string(),
                color: PLASMA_CYAN,
            },
            ColoredMessage {
                text: "🌟 Highlighting your contributions...".to_string(),
                color: AURORA_GREEN,
            },
            ColoredMessage {
                text: "📋 Building your PR description...".to_string(),
                color: SOLAR_YELLOW,
            },
            ColoredMessage {
                text: "🎨 Painting the PR picture...".to_string(),
                color: GALAXY_PINK,
            },
            ColoredMessage {
                text: "🧵 Threading your commits together...".to_string(),
                color: COMET_ORANGE,
            },
        ]
    });

static CHANGELOG_WAITING_MESSAGES: std::sync::LazyLock<Vec<ColoredMessage>> =
    std::sync::LazyLock::new(|| {
        vec![
            ColoredMessage {
                text: "📜 Chronicling your changes...".to_string(),
                color: NEBULA_PURPLE,
            },
            ColoredMessage {
                text: "✨ Cataloging your accomplishments...".to_string(),
                color: STARLIGHT,
            },
            ColoredMessage {
                text: "📖 Writing the history of your code...".to_string(),
                color: CELESTIAL_BLUE,
            },
            ColoredMessage {
                text: "🏛️ Archiving your progress...".to_string(),
                color: PLASMA_CYAN,
            },
            ColoredMessage {
                text: "🔖 Tagging your milestones...".to_string(),
                color: AURORA_GREEN,
            },
            ColoredMessage {
                text: "📝 Documenting the journey...".to_string(),
                color: SOLAR_YELLOW,
            },
            ColoredMessage {
                text: "🗂️ Organizing your achievements...".to_string(),
                color: GALAXY_PINK,
            },
            ColoredMessage {
                text: "⚡ Capturing the deltas...".to_string(),
                color: COMET_ORANGE,
            },
        ]
    });

static RELEASE_NOTES_WAITING_MESSAGES: std::sync::LazyLock<Vec<ColoredMessage>> =
    std::sync::LazyLock::new(|| {
        vec![
            ColoredMessage {
                text: "🚀 Preparing launch notes...".to_string(),
                color: METEOR_RED,
            },
            ColoredMessage {
                text: "✨ Polishing the release highlights...".to_string(),
                color: STARLIGHT,
            },
            ColoredMessage {
                text: "📣 Announcing your achievements...".to_string(),
                color: NEBULA_PURPLE,
            },
            ColoredMessage {
                text: "🎉 Celebrating the release...".to_string(),
                color: AURORA_GREEN,
            },
            ColoredMessage {
                text: "📦 Packaging the release story...".to_string(),
                color: CELESTIAL_BLUE,
            },
            ColoredMessage {
                text: "🌟 Showcasing new features...".to_string(),
                color: PLASMA_CYAN,
            },
            ColoredMessage {
                text: "📢 Composing the release fanfare...".to_string(),
                color: SOLAR_YELLOW,
            },
            ColoredMessage {
                text: "🎊 Wrapping up the release...".to_string(),
                color: GALAXY_PINK,
            },
        ]
    });

fn get_pr_waiting_message() -> ColoredMessage {
    let mut rng = rand::rng();
    PR_WAITING_MESSAGES
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| ColoredMessage {
            text: "Building PR description...".to_string(),
            color: NEBULA_PURPLE,
        })
}

fn get_changelog_waiting_message() -> ColoredMessage {
    let mut rng = rand::rng();
    CHANGELOG_WAITING_MESSAGES
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| ColoredMessage {
            text: "Generating changelog...".to_string(),
            color: CELESTIAL_BLUE,
        })
}

fn get_release_notes_waiting_message() -> ColoredMessage {
    let mut rng = rand::rng();
    RELEASE_NOTES_WAITING_MESSAGES
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| ColoredMessage {
            text: "Creating release notes...".to_string(),
            color: AURORA_GREEN,
        })
}

pub fn get_user_message() -> ColoredMessage {
    let mut rng = rand::rng();
    USER_MESSAGES
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| ColoredMessage {
            text: "What would you like to do?".to_string(),
            color: CELESTIAL_BLUE,
        })
}
