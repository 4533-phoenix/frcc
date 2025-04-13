/*
 * FRCC Card PDF Generator
 */

#set page("us-letter", margin: 0.0in)
#let card_width  = 63mm
#let card_height = 88mm
#let card_pad    = 0.125in

#align(center,
	layout(size => {
		let cards = json("./config.json");
		let page_cols  = int(size.width / (card_width + card_pad));
		let page_rows  = int(size.height / (card_height + card_pad));
		let page_index = 0;

		let cards_per_page = page_cols * page_rows;

		while cards.len() > 0 {
			let card_fronts = ();
			let card_backs  = ();

			for i in range(0, cards_per_page) {
				if cards.len() > 0 {
					let card = cards.pop();
					card_fronts.push(pad(rest: card_pad/4, image(card.front, width: card_width, height: card_height)));
					card_backs.push(pad(rest: card_pad/4, image(card.back, width: card_width, height: card_height)));
				} else {
					break;
				}
			}

			grid(columns: page_cols, rows: page_rows, align: alignment.center,
				..card_fronts
			)
			colbreak();
			grid(columns: page_cols, rows: page_rows, align: center,
				..card_backs
			)
		}
	})
)

