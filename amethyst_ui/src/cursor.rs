//! Module containing the system managing the text editing cursor create, deletion and position.

// TODO: WIP

/// Tag component placed on the cursor of a text field being edited.
pub struct TextEditingCursor;

impl Component for TextEditingCursor {
	type Storage = NullStorage<Self>;
}

/// Manages the text editing cursor create, deletion and position.
pub struct TextEditingCursorSystem;

impl<'a> System<'a> for TextEditingCursorSystem {
	type SystemData = (
		Entities<'a>,
		WriteStorage<'a, UiTransform>,
		ReadStorage<'a, TextEditing>,
		ReadStorage<'a, Parent>,
		ReadStorage<'a, Selected>,
		WriteStorage<'a, Cursor>,
		WriteStorage<'a, Blink>,
		WriteStorage<'a, Handle<Texture>>,
		ReadStorage<'a, UiConfig>,
	);

	fn run(&mut self, (entities, mut transforms, editings, parents, selecteds, mut cursors, mut blinks, mut textures, colors, config): Self::SystemData){
		// Go through all text editing entities.
		for (entity, _) in (&*entities, &editings).join() {

			// Finds child cursor of current text editing entity.
			let cursor = (&*entities, &parents, &cursors).join().filter(|t| t.1.parent == entity).map(|t| t.0).next();
			let selected = selecteds.contains(entity);

			if let Some(c) = cursor {
				if !selected {
					// Shouldn't have a cursor.
					entities.delete(c);
					continue;
				}
			} else {
				if selected {
					// TODO: Should have a cursor.
					let c = entities.create_entity();
					cursors.insert(c, Cursor).expect("Unreachable: Entity just created.");
					parents.insert(c, Parent{parent: entity.clone()}).expect("Unreachable: Entity just created.");
					transforms.insert(c, UiTransform::new()).expect("Unreachable: Entity just created.");
					blinks.insert(c, Blink::new(config.blink_delay)).expect("Unreachable: Entity just created.");
					textures.insert(c, config.cursor).expect("Unreachable: Entity just created.");
				}
			}
			// TODO: Move the cursor to the correct location.
			// TODO: Ajust cursor thicc-ness depending on is block cursor and text char width.

		}
	}
}
