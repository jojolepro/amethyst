
use std::marker::PhantomData;
use amethyst_core::specs::Component;
use amethyst_core::specs::DenseVecStorage;
use amethyst_core::specs::world::EntitiesRes;
use amethyst_core::specs::ReadStorage;
use amethyst_core::specs::Join;
use amethyst_core::specs::SystemData;
use amethyst_core::specs::ReaderId;

use amethyst_renderer::{Event, VirtualKeyCode, WindowEvent, ElementState};

use amethyst_core::specs::Entity;

use amethyst_core::specs::WriteStorage;

use amethyst_core::specs::System;

use std::hash::Hash;

use amethyst_core::specs::Read;

use amethyst_core::shrev::EventChannel;

use {CachedSelectionOrder, UiEvent, UiEventType};

use amethyst_input::InputHandler;

use amethyst_renderer::KeyboardInput;

use amethyst_core::specs::Resources;

/// Component indicating that a Ui entity is selectable.
/// Generic Type:
/// - G: Selection Group. Used to determine which entities can be selected together at the same time.
#[derive(Debug, Serialize, Deserialize, new)]
pub struct Selectable<G> {
	pub order: u32,
	#[new(default)]
	pub multi_select_group: Option<G>,
	#[new(default)]
	pub auto_multi_select: bool,
	/// Indicates if this requires the inputs (except Tab) be ignored when the component is focused.
	#[new(default)]
	pub require_input: bool,
}

impl<G: Send + Sync + 'static> Component for Selectable<G> {
	type Storage = DenseVecStorage<Self>;
}

/// Component indicating that a Ui entity is currently selected.
#[derive(Debug, Serialize, Deserialize)]
pub struct Selected;

impl Component for Selected {
	type Storage = DenseVecStorage<Self>;
}

/// System managing the selection of entities.
/// Reacts to `UiEvent`.
/// Reacts to Tab and Shift+Tab.
#[derive(Debug, Default, new)]
pub struct SelectionSystem<G, AX, AC> {
	#[new(default)]
	ui_reader_id: Option<ReaderId<UiEvent>>,
	#[new(default)]
	window_reader_id: Option<ReaderId<Event>>,
	phantom: PhantomData<(G, AX, AC)>,
}

impl<'a, G, AX, AC> System<'a> for SelectionSystem<G, AX, AC> 
where
	G: Send + Sync + 'static + PartialEq,
	AX: Hash + Eq + Clone + Send + Sync + 'static,
	AC: Hash + Eq + Clone + Send + Sync + 'static,
{
	type SystemData = (
		Read<'a, EventChannel<UiEvent>>,
		Read<'a, EventChannel<Event>>,
		Read<'a, CachedSelectionOrder>,
		WriteStorage<'a, Selected>,
		ReadStorage<'a, Selectable<G>>,
		Read<'a, InputHandler<AX, AC>>,
	);
	fn run(&mut self, (ui_events, window_events, cached, mut selecteds, selectables, input_handler): Self::SystemData) {
		/*
		Add clicked elements + shift + ctrl status.
		If tab or shift-tab
			remove clicked buf
			add replace: select higher or lower id closes to previous highest old id
		if clicked buf isn't empty
			if check currently highest selected multiselect group
				// if shift && ctrl -> shift only
				if shift
					add multiple
				else if ctrl || auto_multi_select
					add single
				else
				    add replace
			else
				add replace
		*/


		let mut clicked_buf = vec![]; // Last = last selected
		let mut shift = input_handler.key_is_down(VirtualKeyCode::LShift) || input_handler.key_is_down(VirtualKeyCode::RShift);
		let mut ctrl = input_handler.key_is_down(VirtualKeyCode::LControl) || input_handler.key_is_down(VirtualKeyCode::RControl);

		// Add clicked elements to clicked buffer
		for ev in ui_events.read(self.ui_reader_id.as_mut().unwrap()) {
			match ev.event_type {
				UiEventType::ClickStart => {
					// Ignore events from elements removed between the event emission and now.
					if selectables.get(ev.target).is_some() {
						clicked_buf.push(ev.target);
					}
				},
				_ => {},
			}
		}

		// Checks if tab was pressed.
		// TODO: Controller support
		for event in window_events.read(self.window_reader_id.as_mut().unwrap()) {
            match *event {
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Tab),
                                    modifiers,
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                	// If we press tab, we remove everything that was previously selected.
                	clicked_buf = vec![];
                	if modifiers.shift {
                		shift = true;
                	}
                	if modifiers.ctrl {
                		ctrl = true;
                	}

                	// Get index of highest selected ui element
                	let highest = cached.highest_order_selected_index(&selecteds);

                	if let Some(highest) = highest {
                		// If Some, an element was currently selected. We move the cursor to the next or previous element depending if Shift was pressed.
                		// Select Replace
	                	selecteds.clear();

	                	let target = if !shift {
	                		// Up
	                		cached.cache.get(highest - 1).unwrap_or(cached.cache.last()
	                			.expect("unreachable: A highest ui element was selected, but none exist in the cache."))
	                	} else {
	                		// Down
	                		cached.cache.get(highest + 1).unwrap_or(cached.cache.first()
	                			.expect("unreachable: A highest ui element was selected, but none exist in the cache."))
	                	};
	                	selecteds.insert(target.1, Selected).expect("unreachable: We are inserting");
                	} else {
                		// If None, nothing was selected. Try to take lowest if it exists.
                		if let Some(lowest) = cached.cache.first() {
                			selecteds.insert(lowest.1, Selected).expect("unreachable: We are inserting");
                		}
                	}
                },
                _ => {},
            }
        }
        if !clicked_buf.is_empty() {
        	for clicked in clicked_buf {
        		// Inside of the loop because its possible that the user clicks two times in a frame while pressing shift.
        		let highest = cached.highest_order_selected_index(&selecteds);

	        	if let Some(highest) = highest {
	        		// Safe unwraps, we just got those values from the cache.

	        		let (highest_is_select, auto_multi_select) = {
	        			let highest_multi_select_group = &selectables.get(cached.cache.get(highest).unwrap().1).unwrap().multi_select_group;

		        		let (target_multi_select_group, auto_multi_select) = {
		        			// Safe unwrap because when filing the buffer we checked that the component still exist on the entity.
		        			let target_selectable = selectables.get(clicked).unwrap();
		        			(&target_selectable.multi_select_group, target_selectable.auto_multi_select)
		        		};
		        		(highest_multi_select_group == target_multi_select_group, auto_multi_select)
	        		};

	        		if highest_is_select {
		        		if shift {
		        			// Add from latest selected to target for all that have same multi_select_group
		        			let cached_index_clicked = cached.index_of(clicked)
		        				.expect("unreachable: Entity has to be in the cache, otherwise it wouldn't have been added.");

		        			// When multi-selecting, you remove everything that was previously selected, and then add everything in the range.
		        			selecteds.clear();

		        			let min = cached_index_clicked.min(highest);
		        			let max = cached_index_clicked.max(highest);

		        			for i in min..=max {
		        				let target_entity = cached.cache.get(i).expect("unreachable: Range has to be inside of the cache range.");
		        				selecteds.insert(target_entity.1, Selected).expect("unreachable: We are inserting");
		        			}
		        		} else if ctrl || auto_multi_select {
		        			// Select adding single element
		        			selecteds.insert(clicked, Selected).expect("unreachable: We are inserting");
		        		} else {
		        			// Select replace, because we don't want to be adding elements.
		        			selecteds.clear();
		        			selecteds.insert(clicked, Selected).expect("unreachable: We are inserting");
		        		}
		        	} else {
		        		// Different multi select group than the latest one selected. Execute Select replace
		        		selecteds.clear();
		        		selecteds.insert(clicked, Selected).expect("unreachable: We are inserting");
		        	}
	        	} else {
	        		// Nothing was previously selected, let's just select single.
	        		selecteds.insert(clicked, Selected).expect("unreachable: We are inserting");
	        	}
        	}
        }

	}

	fn setup(&mut self, res: &mut Resources) {
		Self::SystemData::setup(res);
		self.ui_reader_id = Some(
			res.fetch_mut::<EventChannel<UiEvent>>().register_reader()
		);
		self.window_reader_id = Some(
			res.fetch_mut::<EventChannel<Event>>().register_reader()
		);
	}
}