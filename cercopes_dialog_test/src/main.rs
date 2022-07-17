use std::sync::Arc;
use const_random::const_random;
use rand::{distributions::Standard, prelude::*};
use cercopes_dialog::*;
use cercopes_knowledge::*;
use character::Character;

mod dialog;
mod frame;
mod ontology;
mod character;
mod grammar;

pub struct DialogTypes;

impl DialogTrait for DialogTypes {
    type Topic = dialog::Topic;

    type DialogMove = dialog::DialogMove;

    type ExpanderNode = dialog::ExpanderNode;

    type Character = (Arc<Character>, Box<MentalModel<ontology::KnowledgeTypes>>); 
}

type MoveNode = cercopes_dialog::MoveNode<DialogTypes>;
type Manager = DialogManager<DialogTypes>;
type Frame = goal::Frame<DialogTypes>;
type Goal = goal::Goal<DialogTypes>;
type ConversationState = cercopes_dialog::Conversation<DialogTypes>;
type GoalMove = goal::GoalMove<DialogTypes>;

fn finish_conversation(
    manager: &Manager,
    conversation: &mut ConversationState,
) {
    while !conversation.done {
        manager.step_conversation(conversation, &Default::default()).unwrap();
    }
}

fn print_conversation(conversation: &ConversationState) {
    let person0_name = conversation.person0.character.0.name.first;
    let person1_name = conversation.person1.character.0.name.first;

    for hmove in conversation.history.iter() {
        let name = if hmove.speaker == Speaker::Person0 {
            person0_name
        } else {
            person1_name
        };
        println!("{name}: {}", hmove.utterance);
    }
}

fn print_strongest_beliefs(
    conversation: &ConversationState,
    facets: &[ontology::TestFacet],
) {
    let (ref person0, ref mental_model01) = conversation.person0.character;
    let (ref person1, ref mental_model10) = conversation.person1.character;
    println!("\n{}'s beliefs about {}:", person0.name.first, person1.name.first);
    for facet in facets {
        let belief = if let Some(belief) = mental_model01.get_strongest_belief(facet) {
            belief.to_string()
        } else {
            "none held".to_string()
        };
        println!("{facet}: {belief} (truth: {})", person1.facet_truth(facet).unwrap());
    }
    println!("\n{}'s beliefs about {}:", person1.name.first, person0.name.first);
    for facet in facets {
        let belief = if let Some(belief) = mental_model10.get_strongest_belief(facet) {
            belief.to_string()
        } else {
            "none held".to_string()
        };
        println!("{facet}: {belief} (truth: {})", person0.facet_truth(facet).unwrap());
    }
}

fn bench(
    manager: &DialogManager<DialogTypes>,
    initiator: Arc<Character>,
    recipient: Arc<Character>,
    ir_model: &Box<MentalModel<ontology::KnowledgeTypes>>,
    ri_model: &Box<MentalModel<ontology::KnowledgeTypes>>,
    lull: rand::distributions::Bernoulli,
) {
    let timeout = std::time::Duration::new(10, 0);
    let mut total = 0_u64;
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        for _ in 0..1_000 {
            let mut conversation = manager.new_conversation(
                Speaker::Person0,
                lull,
                (initiator.clone(), ir_model.clone()),
                (recipient.clone(), ri_model.clone()),
            );
            finish_conversation(&manager, &mut conversation);
        }

        total += 1_000;
    }

    println!("Average speed: {}/second", total as f32 / start.elapsed().as_secs_f32());
}

fn rust_test() {
    println!("Starting Rust test:");
    let mut rng = thread_rng();
    let manager = Manager::new(
        dialog::manager_nodes(),
        frame::manager_frames(),
    );
    let initiator = Arc::new(rng.gen::<Character>());
    let recipient = Arc::new(rng.gen::<Character>());
    let initiator_model = Box::new(MentalModel::new(initiator.clone(), recipient.clone()));
    let recipient_model = Box::new(MentalModel::new(recipient.clone(), initiator.clone()));
    println!("{initiator}");
    println!("{recipient}");
    let mut conversation = manager.new_conversation(
        Speaker::Person0,
        rand::distributions::Bernoulli::new(0.2).unwrap(),
        (initiator.clone(), initiator_model.clone()),
        (recipient.clone(), recipient_model.clone()),
    );
    finish_conversation(&manager, &mut conversation);
    print_conversation(&conversation);
    print_strongest_beliefs(&conversation, &[
        ontology::TestFacet::FavMusicGenre,
    ]);
    bench(
        &manager,
        initiator,
        recipient,
        &initiator_model,
        &recipient_model,
        rand::distributions::Bernoulli::new(0.5).unwrap(),
    );
}

fn lua_test() -> mlua::Result<()> {
    println!("Starting Lua test:");
    let lua = cercopes_mlua::create_lua()?;
    let start = std::time::Instant::now();
    static SOURCE: &str = include_str!("test.lua");
    lua.load(SOURCE).exec()?;
    print!("Executed in {:.1}ms", start.elapsed().as_secs_f32() * 1_000.0);
    Ok(())
}

fn main() {
    rust_test();
    lua_test().unwrap();
}
