
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PrettySpineItem {
    pub index: usize,
    pub number: usize,
    pub title: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChapterResponse {
    pub chapter_index: usize,
    pub num_chunks: usize,
    pub text: String,
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CursorTextResponse {
    pub cursor: domain::cursor::BookCursor,
    pub text: String, // 
}


use crate::{domain, infra::auth::get_with_auth};
#[cfg(not(feature = "mock"))]
pub async fn fetch_book_nav(book_id: &str) -> Result<Vec<PrettySpineItem>, String> {
    let url = format!("/api/v1/books/{}/nav", book_id);

    let resp = get_with_auth(&url)
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to fetch chapters for {}: {}", book_id, resp.status()).into());
    }

    let chapters: Vec<PrettySpineItem> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(chapters)
}


#[cfg(feature = "mock")]
pub async fn fetch_book_nav(book_id: &str) -> Result<Vec<PrettySpineItem>, String> {
    use serde_json::json;
    use serde_json::Value;

    // JSON data per book_id
    let json_data: Value = match book_id {
        "b1" => json!([
            { "index": 4, "number": 1, "title": "Chapter: 1 New Beginnings" },
            { "index": 5, "number": 2, "title": "Chapter: 2 The Caravanner’s Guild" },
            { "index": 6, "number": 3, "title": "Chapter: 3 Dinner" },
            { "index": 7, "number": 4, "title": "Chapter: 4 A Simple Home" }
        ]),
        "b2" => json!([
            { "index": 0, "number": 1, "title": "First Chapter" },
            { "index": 1, "number": 2, "title": "Second Chapter" }
        ]),
        _ => json!([
            { "index": 0, "number": 1, "title": "Default Chapter" }
        ]),
    };

    // Deserialize JSON into Vec<PrettySpineItem>
    let chapters: Vec<PrettySpineItem> = serde_json::from_value(json_data)
        .map_err(|e| format!("Failed to deserialize mock JSON: {}", e))?;

    Ok(chapters)
}
#[cfg(not(feature = "mock"))]
pub async fn fetch_chapter(book_id: &str, chapter_index: usize) -> Result<String, String> {
    let url = format!("/api/v1/books/{}/chapters/{}", book_id, chapter_index);

    let resp = get_with_auth(&url)
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!(
            "Failed to fetch chapter {} for {}: {}",
            chapter_index,
            book_id,
            resp.status()
        ));
    }

    // Read the response as plain text
    let chapter_text = resp.text().await.map_err(|e| e.to_string())?;

    Ok(chapter_text)
}
#[cfg(feature = "mock")]
pub async fn fetch_chapter(book_id: &str, chapter_index: usize) -> Result<String, String> {
    // Paste your mock data here
    let chapter = match (book_id, chapter_index) {
        ("b1", 0) => "<div class=\"heading_s2u\"><p class=\"class_s2p\">Chapter: 1</p><p class=\"class_s2s\"><span class=\"class_s5s\">New Beginnings</span></p></div><p class=\"class_sh\">Frost licked over Tala’s already sensitive skin, accompanied by the static tension of power rippling through her from an outside source.</p><p class=\"class_sh\">With a pulse of darkness, she left her old life, her adolescence of learning and exploration, behind.</p><p class=\"class_sh\">She crouched low in the center of a large, white-speckled, granite room. It was the shape of a half-sphere, each block sculpted and placed so precisely that had she not known better, she’d have believed it was carved from a single piece.</p><p class=\"class_sh\"><span class=\"class_s5rm\">Though, I suppose a Material Creator could have summoned the room into being, fully formed.</span> That was unlikely. If her schooling had taught her anything, it was that magic was expensive; why would anyone do something with it, which could be done by hand?</p><p class=\"class_sh\">Beneath her were the empty grooves of a spell-form, an anchor used to draw a target in and recombine them.</p><p class=\"class_sh\">Everyone said teleportation was tricky, and that was true, in part. Disintegration and expulsion of a person was incredibly simple. Calling that person, and all their requisite pieces, back from the ether and putting them all back where they belonged, now that was tricky business.</p><p class=\"class_sh\">She shivered, as much from the fading cold as from the existentially terrifying thoughts. <span class=\"class_s5rm\">A person’s soul does most of the work, Tala. It’s not like the scripts could get your insides wrong.</span></p><p id=\"page_7\" class=\"class_sh\">She glanced down at her hands and saw fading red traces where her spell-lines should have been. She let out a short groan. <span class=\"class_s5rm\">Well, that didn’t work…</span></p><p class=\"class_sh\">Blessedly, she saw her own dark hair, roughly shoulder length, swaying in her peripheral vision. The inscribers at the academy shaved all the students’ heads to allow for the easier adding of spell-lines, but in her soul—how she viewed herself—Tala had hair. Thus, somehow, her recombination had returned it to her. <span class=\"class_s5rm\">Now, I just have to find an inscriber capable of leaving it be.</span></p><p class=\"class_sh\"><span class=\"class_s5rm\">Huh… my skin is still raw.</span><span class=\"class_s5rm\">Shouldn’t it be as healed and complete as my hair?</span> She supposed that some things just didn’t make sense.</p><p class=\"class_sh\">Tala heard several of the guards gasp as one voice stuttered out, “She’s… She’s naked!”</p><p class=\"class_sh\">A commanding voice cracked out. “Go check her! If the teleportation acolytes at the academy managed to leave her clothes behind, who knows what else was forgotten.”</p><p class=\"class_sh\"><span class=\"class_s5rm\">Take charge of your life, Tala.</span> She sighed, standing fully upright, back straight.</p><p class=\"class_sh\">An uninscribed guard, a tall, broad-shouldered and grizzled man, stepped back in surprise at the sudden movement.</p><p class=\"class_sh\">Tala looked around the room, ignoring the man. A waist-high stone wall stood in a circle halfway between her and the smooth granite of the outer walls. It was broken only in one place, allowing access to the inner circle.</p><p class=\"class_sh\">Everyone—six guards and two Mages—was staring at her.</p><p class=\"class_sh\">One of the Mages, heavens bless him, was coloring so that the red was easily visible, even under his spell-lines. He was sparsely clad, as befit an on-duty Mage, and he was, somehow, blushing nearly down to his navel.</p><p id=\"page_8\" class=\"class_sh\">Tala cleared her throat, speaking softly but letting her voice carry. “Nothing’s for sale, gents, so please stop window shopping.”</p><p class=\"class_sh\">Three of the guards turned away, blushing in turn. The two others grinned but averted their eyes. The one already in the circle with her huffed something near a laugh but turned slightly away, keeping his eyes to himself.</p><p class=\"class_sh\">That poor mageling flushed even redder and turned, putting his face against the outer wall. The female Mage, likely his sponsor, rolled her eyes and walked forward with a blanket taken from a pile that rested on a shelf laden with supplies.</p><p class=\"class_sh\">She was practically naked herself, cloth covering as little as possible, while maintaining the semblance of modesty. Her lines were proudly on display, their magic unhindered by covering. She was not young, but wrinkles had yet to render her inscriptions faulty. Both Mages were fit, if not well-muscled—as most Mages had to be. Changing size or shape would almost universally ruin your spell-lines, as well as force your inscriber to rebuild your spell-work from scratch. That was assuming the distortions didn’t make such work impossible.</p><p class=\"class_sh\">Make no mistake, Mages, one and all, were vain creatures, but it wasn’t their vanity that inspired scrupulous attention to their own bodies, so much as devotion to their art.</p><p class=\"class_sh\">The older Mage moved with practiced grace and fluidity, obviously aware of her every gesture, careful not to brush any of her lines against others. Such contact would usually be safe, but so would juggling knives; it was the unexpected that killed, and when spell-lines were involved, there was far more than a cut hand on the line.</p><p class=\"class_sh\">The older guard walked beside her as Tala strode to meet the Mage. If she had to guess, he had strategically placed himself between her and some of the other guards, blocking their view of her. <span class=\"class_s5rm\">Thoughtful of him.</span></p><p id=\"page_9\" class=\"class_sh\">A furnace blazed on the opposite side of the room, and its heat was slowly taking the teleportation chill from her. <span class=\"class_s5rm\">Quickly, now. Don’t let them see how embarrassed you are.</span> She found herself blessing the chill, which had kept the flush from the surface.</p><p class=\"class_sh\">As the Mage drew close, she lowered her tone to keep it from carrying. “The chill does many things, dear, but it doesn’t hide <span class=\"class_s5rm\">every</span> sign of your embarrassment, at least not from those who know to look.” She draped the blanket over Tala’s shoulders. “Now, how did you arrive in such a state?” She frowned. “Why does it look like someone put you through a sandblaster? You’ve raw, new skin across your whole body.”</p><p class=\"class_sh\">Tala gave a formal half-bow, clutching the blanket close, while trying to affect a nonchalance that she did not feel. Though it was soft, the blanket still chafed lightly on her skin. The rawness had little to do with the unclad teleport, though it was still her own doing. “I’m Tala, Mistress, newly graduated from the academy.”</p><p class=\"class_sh\">“Yes, dear. You may call me Phoen. You have not answered my questions.”</p><p class=\"class_sh\">Tala cleared her throat, glancing away. “Well, you see, Mistress Phoen. Our current teleportation spells strip away spell-lines and won’t take any gear, save the clothes on your back.”</p><p class=\"class_sh\">“Hmmm?”</p><p class=\"class_sh\">“In studying the formula, it looked like it might be some factor of mass, beyond the organic being teleported, that is why at least a modicum of clothing always comes. Metal only comes if the person was wearing armor, and then not very much of it.”</p><p class=\"class_sh\">Phoen sighed. “So, you thought to, what? Modify the spell somehow? Child, you are lucky you didn’t scatter yourself across half of inner-solar space!”</p><p class=\"class_sh\">Tala’s eyes widened. “Oh, no! Absolutely not!”</p><p class=\"class_sh\">Phoen narrowed her eyes. “Then, what?”</p><p id=\"page_10\" class=\"class_sh\">“I guessed that, without clothes to teleport, other material would be brought along.” She held up her hands. The red marks were already faded into bare visibility. “But I missed something.”</p><p class=\"class_sh\">“…Wait…”</p><p class=\"class_sh\">“Hmmm?”</p><p class=\"class_sh\">“Do you mean to tell me that you went <span class=\"class_s5rm\">into</span> the teleportation circle… naked?”</p><p class=\"class_sh\">Tala cleared her throat and looked away. As she did so, she was able to see two guards using heavy metal tongs to move a crucible from the furnace to the short wall. They then poured the contents, liquid silver, down a funnel set into that stone.Tala cleared her throat and looked away. As she did so, she was able to see two guards using heavy metal tongs to move a crucible from the furnace to the short wall. They then poured the contents, liquid silver, down a funnel set into that stone.Tala cleared her throat and looked away. As she did so, she was able to see two guards using heavy metal tongs to move a crucible from the furnace to the short wall. They then poured the contents, liquid silver, down a funnel set into that stone.Tala cleared her throat and looked away. As she did so, she was able to see two guards using heavy metal tongs to move a crucible from the furnace to the short wall. They then poured the contents, liquid silver, down a funnel set into that stone.Tala cleared her throat and looked away. As she did so, she was able to see two guards using heavy metal tongs to move a crucible from the furnace to the short wall. They then poured the contents, liquid silver, down a funnel set into that stone.Tala cleared her throat and looked away. As she did so, she was able to see two guards using heavy metal tongs to move a crucible from the furnace to the short wall. They then poured the contents, liquid silver, down a funnel set into that stone.Tala cleared her throat and looked away. As she did so, she was able to see two guards using heavy metal tongs to move a crucible from the furnace to the short wall. They then poured the contents, liquid silver, down a funnel set into that stone.Tala cleared her throat and looked away. As she did so, she was able to see two guards using heavy metal tongs to move a crucible from the furnace to the short wall. They then poured the contents, liquid silver, down a funnel set into that stone.</p><p class=\"class_sh\">She knew the formulas needed for this spell-form well. <span class=\"class_s5rm\">Precisely two pounds of silver.</span></p><p class=\"class_sh\">The metal flowed out of a spout low in the wall and washed through the grooved lines of the spell form, which was set into the floor.</p><p class=\"class_sh\">She didn’t know what preparations had been laid into the stone to ensure the silver would always distribute evenly and cleanly. She hadn’t studied the Builder Arts, after all. Nonetheless, the Mages’ work was flawless, and the spell-form was filled once more, allowing the silver to cool evenly, creating strong, solid spell-lines.</p><p class=\"class_sh\">Tala had found variations of this catching spell that used a combination of metals, thus making them much more efficient from the perspective of materials, but the difficulty in casting interlacing liquids quickly meant that the uniform version was vastly easier to use, and thus the most pervasive.</p><p class=\"class_sh\">Phoen sighed. “Mact!”</p><p class=\"class_sh\">The young mageling jumped, turning around. “Mistress?”</p><p class=\"class_sh\">“The spell-lines are reset. Take your place.”</p><p class=\"class_sh\">“Yes, Mistress!” He scurried around the women and went to sit in the center of the spell-lines, a hand resting within hand-<span id=\"page_11\"></span>sized outlines to either side of him. He sat straight, his core tight, his limbs carefully aligned. He took a deep breath and exhaled.</p><p class=\"class_sh\">Tala felt the power ripple out from the boy, activating and resetting this teleportation receiver.</p><p class=\"class_sh\">Without delay, Mact stood and returned to his master.</p><p class=\"class_sh\">“Well done, Mact.”</p><p class=\"class_sh\">“Thank you.” He smiled happily, almost to himself.</p><p class=\"class_sh\">“Now, girl. You are beginning to tire me.”</p><p class=\"class_sh\">Tala sighed. “Yes, I went into the circle naked. Yes, I was lectured by the Mages on the other end about the folly of it. Yes, I know that teleportation magic isn’t intended to work on naked subjects.” She pulled the blanket closer together in front, and the top billowed out slightly, causing it to fall from her shoulders, exposing her back.</p><p class=\"class_sh\">The grizzled guard let out a little startled exhalation, then started to laugh.</p><p class=\"class_sh\">Tala spun on him. “What’s so funny?”</p><p class=\"class_sh\">Phoen let out a similar sound and barked a laugh of her own.</p><p class=\"class_sh\">Tala turned back. “Mistress Phoen?”</p><p class=\"class_sh\">“You seem to be cleverer than I’d thought.” After a moment’s pause, she amended, “Or, your cleverness bore more fruit than we’d guessed.”</p><p class=\"class_sh\">Tala frowned. Then, her eyes widened in realization. “My keystone?”</p><p class=\"class_sh\">“Yes, your keystone looks intact. Come, I’ll examine it.”</p><p class=\"class_sh\">Tala thanked the guard and followed Phoen from the room.</p><p class=\"class_sh\">Mact tried to follow, but Phoen sent him back with several stern words.</p><p class=\"class_sh\">Less than two minutes later, Tala was sitting in a small side room, a blanket covering herself strategically while leaving her back exposed. She was naturally straight-backed, her feet flat on the floor, knees bent at as close to right angles as the seat allowed—as she’d been trained.</p><p id=\"page_12\" class=\"class_sh\">Phoen took nearly five minutes examining the spell-lines in excruciating detail. “Child, what type of Mage are you?”</p><p class=\"class_sh\">“Immaterial Guide, Mistress.”</p><p class=\"class_sh\">She grunted. “That explains it. I’m a Material Creator. None of these mean a thing to me. Though, they do look intact. You’ll need an inscriber to look these over.” She sighed. “Fresh from the academy, right?”</p><p class=\"class_sh\">“Yes.”</p><p class=\"class_sh\">“If you’re here, I assume you’ve signed a contract with the Caravanners, or maybe the Constructionists or Wainwrights? Though, I didn’t think the latter two took on magelings, here…”</p><p class=\"class_sh\">Tala grinned. “Not yet.”</p><p class=\"class_sh\">Phoen blinked at her, cocked her head to one side, and then sighed. “Oh, child.”</p><p class=\"class_sh\">“What? It’s the law.”</p><p class=\"class_sh\">“<span class=\"class_s5rm\">If</span> that inscription is still viable, you have a case, but they may not be happy about it. They might just turn you away.”</p><p class=\"class_sh\">“I…” Tala hadn’t thought of that. Magelings got such poor pay until they could buy some spell-lines themselves. In addition, they had to operate under a full Mage, bound to obey them, subject to their schedule and whims. Once the mageling had scraped together enough to afford their own spell-lines, though, they were a Mage, and it was common law that a Mage commanded a much higher salary. She’d not considered that, given a choice of paying her a high salary or not hiring her, they might simply not hire her. She cursed.</p><p class=\"class_sh\">Phoen quirked a small smile. “You must have been a joy to your teachers.”</p><p class=\"class_sh\">Tala bristled. “My teachers loved me.” After a moment, she amended, “Most of them, anyways.”</p><p class=\"class_sh\">Phoen just grinned.</p><p class=\"class_sh\">“Well, what can I do?”</p><p id=\"page_13\" class=\"class_sh\">“You have to decide whether or not to gamble. Don’t tell them you have inscriptions until after the contract is signed and accept the lower wage; or tell them, and possibly lose any chance at work. No one else is hiring those of your quadrant in this city… that I know of.” She smiled ruefully. “If you were a Material Creator, I’d throw you out on your ear for hubris.” Even so, her eyes twinkled. “But not everyone’s as crotchety as I. Perhaps you’ll be lucky.”</p><p class=\"class_sh\">Tala frowned. “So, I’m naked, likely for nothing… Lovely.”</p><p class=\"class_sh\">Phoen opened her mouth to comment, but Tala held up a hand.</p><p class=\"class_sh\">“Please… I know I’m asking for it, but please don’t.”</p><p class=\"class_sh\">Phoen patted her on the shoulder. “I’ll get you some clothes, dear. I have a friend who’s an inscriber, and she should be able to verify your spell-lines. Then, you can make your own choice.”</p><p class=\"class_sh\">“Thank you… for everything.”</p><p class=\"class_s5h\">*<span class=\"class_s5rr\"></span>*<span class=\"class_s5rs\"></span>*</p><p class=\"class_sv\">Half an hour later, Tala was dressed in surprisingly soft, simple clothes and heading out of the great doors, several floors below the teleportation receiving areas.</p><p class=\"class_sh\">She wore no shoes for two reasons. First, shoes were expensive and should be custom-made to be more help than harm. Second, some Mages preferred going barefoot, and in this, Tala’s oddities were no exception.</p><p class=\"class_sh\">Phoen’s inscriber had verified that Tala’s keystone spell-lines were intact and functional. Blessedly, the trickiest portion of her inscriptions had been maintained.</p><p class=\"class_sh\">While most spell-lines were scripted thin to avoid interference, the keystone was always made as robust as possible. As a result, the keystone only had to be refreshed every year or so, with normal casting. Heavy casters still only had to have that work redone every six months, at the most often.</p><p id=\"page_14\" class=\"class_sh\">In contrast, the ancillary spell-lines could be used up in days—faster with heavy casting. Even standard amounts of magical work forced many inscriptions to be refreshed every couple of weeks.</p><p class=\"class_sh\">As a result, the work and materials required for the keystone were tremendous. In general, Mages spent as much on the once- or twice-a-year keystone work as on all the ancillary inscriptions for the rest of the year combined. In many cases, the keystone work could cost as much as two years of ancillary lines.</p><p class=\"class_s5u\">Ahh, math. How I hate how much I need thee.</p><p class=\"class_sh\">She paused before exiting the tower fully, taking a moment to admire the craftsmanship of the arch and doors that stood open, allowing entrance into the teleportation tower. <span class=\"class_s5rm\">Magic rarely makes beauty.</span> And the beauty of this work spoke of human labor.</p><p class=\"class_sh\">Tala shook her head. <span class=\"class_s5rm\">I can’t imagine striving to add embellishments to buildings that won’t last even four centuries.</span> Even so, she enjoyed them. She idly wondered how many passersby had already gained a measure of pleasure from the elaborations. <span class=\"class_s5rm\">Maybe, that’s enough.</span></p><p class=\"class_sh\">Turning her gaze outward, she looked out on Bandfast for the first time.</p><p class=\"class_sh\">The sky above the city was the deep blue of a clear autumn day, with a scattering of thin, high clouds. She loved such days, such skies.</p><p class=\"class_sh\">Below the clear blue beauty, from this high vantage, she could easily see six layers of the city’s defenses. All but the outermost were still in place, making the burgeoning nature of the city even more apparent. <span class=\"class_s5rm\">It’s in the farming phase.</span></p><p class=\"class_sh\">Indeed, the city’s outermost active defenses encompassed vast tracts of farmland. Those defensive scripts were enormously taxing and would only last for the first hundred and fifty years <span id=\"page_15\"></span>of a city’s life. By the growth on the land, the city was close to halfway between leaving the first and entering the third phase.</p><p class=\"class_sh\">The only ring beyond the farmland was the mines, but those would have been abandoned in this second phase city, their defenses already depleted.</p><p class=\"class_sh\">When the farmland’s defenses faltered, the workers would move inward to the foundries, ore processing plants, and raw-goods refineries of the third ring.</p><p class=\"class_sh\">Inside of that were factories, workshops, and artisan shops, which stood ready within the next layer of defenses.</p><p class=\"class_sh\">The next layer contained the clerks and organizers of the city.</p><p class=\"class_sh\">Inside that, the final layer of defenses held the homes and services like the teleportation tower.</p><p class=\"class_sh\">The fifth phase of every city simply allowed for the buttoning up of all loose ends, and the sixth kept those remaining people comfortable as they prepared to leave and then left. She’d heard mention of other tasks and opportunities surrounding the final years of a waning city, but had never delved too deeply. As a new Mage, she knew better than to consider work for the Harvesters Guild, at least for now.</p><p class=\"class_sh\">One hundred years of mining, an additional fifty years of farming, fifty more of refining, fifty of manufacturing, then twenty-five years each of closing down and departing.</p><p class=\"class_sh\">Three hundred years: the lifespan of a city, with only the last twenty-five years of waning to lament the end.</p><p class=\"class_sh\">All of this to keep humanity safe.</p><p class=\"class_sh\">As if on cue, she felt a thrum of power and saw a lance of lightning strike from one of the outermost towers into the sky. The piercing scream of an eagle split the air, despite the great distance, and she was able to see the great beast spiraling downward to crash into some poor farmer’s field. <span class=\"class_s5rm\">Not too poor. </span>That large corpse would bring substantial payment to the one <span id=\"page_16\"></span>who had lucked into receiving it. <span class=\"class_s5rm\">Assuming it didn’t drop on their heads.</span></p><p class=\"class_sh\">She sighed, contemplating the slain creature. <span class=\"class_s5rm\">I have not missed that.</span> The academy, for some inexplicable reason, did not have to deal with arcanous or magical beasts. <span class=\"class_s5rm\">Yet more unknowns.</span></p><p class=\"class_sh\">Tala shook her head, coming back from her reverie. <span class=\"class_s5rm\">This city still has at least a hundred and fifty years. </span>Probably closer to two hundred, if she had to guess. She would be long dead before it was fully abandoned. <span class=\"class_s5rm\">Unless I go back to the academy…</span></p><p class=\"class_sh\">For reasons that no one had been able to explain to her, the longer someone stayed at the academy, the slower they aged, but also the weaker their abilities with magic became. Finally, after endless pestering, Tala had determined that even the faculty had no idea why it worked as it did.</p><p class=\"class_sh\">She smiled to herself, realizing that she’d fallen back into musings. <span class=\"class_s5rm\">To the Caravanner’s main office.</span> That would be in the ring one out from where she stood, with the other bureaucratic and guild offices.</p><p class=\"class_sh\">The inscribers would be here, in the innermost ring, and she itched to have her spell-lines refreshed, but she lacked the funds to pay for such services. Like most students, she left the academy not with accounts bursting, but indebted to the institution for her training. She, herself, had… other debts, as well.</p><p class=\"class_s5u\">I’m delaying again.</p><p class=\"class_sh\">With no further introspections, she strode through the archway and down the front steps, allowing herself to enjoy the artistry of the carvings as she passed.</p><p class=\"class_sh\">The streets were busy but nowhere near capacity. After all, this section contained the housing for nearly the city’s entire population—as well as several of the smaller market areas—and had been built accordingly. The majority of the population <span id=\"page_17\"></span>would be about their work—mostly farming, given the city’s phase.</p><p class=\"class_sh\">Even so, the streets were far from empty.</p><p class=\"class_sh\">Several large arcanous animals trudged through the streets, led by handlers. There were oxen, whose shoulders stood twice her height; horses, both massive and diminutive, pulling loads that seemed comically overlarge for them; and even several clearly arcanous pets padding alongside their owners. In every case, a simple scripted collar enclosed the arcanous animal’s neck, denoting them as tamed or domesticated, exempting them from the city’s defensive magics.</p><p class=\"class_sh\">Thankfully, Mages didn’t need to wear any such thing, as human magic seemed to function differently enough that wards could differentiate.</p><p class=\"class_sh\">As her eyes scanned those she passed, she was able to pick out the occasional Mage by their bearing and fluid manner of movement, not to mention the spell-lines evident across their exposed skin. Most also wore Mage’s robes, but not all.</p><p class=\"class_sh\">To her surprise, she also saw an arcane, a humanoid arcanous creature.</p><p class=\"class_sh\">What had caught her attention at first was the leather collar he wore, though it was tucked low, almost entirely hidden by his shirt’s collar. As she’d looked closer, ensuring that her eyes hadn’t deceived her and that it wasn’t just an odd fashion choice, he’d turned to regard her. She hadn’t noticed his gaze until after she’d seen the metallic spell-lines on the leather collar.</p><p class=\"class_sh\">When she had felt his gaze, her eyes flicked up, meeting his, and she felt frozen to the spot.</p><p class=\"class_sh\">His eyes were blood.</p><p class=\"class_sh\">No comparison held the weight of truth save to say that his eyes were spheres of fresh, liquid blood, unbroken save for small circular scabs in place of pupils.</p><p id=\"page_18\" class=\"class_sh\">Tala swallowed involuntarily. <span class=\"class_s5rm\">He’s looking at me.</span> She tried to smile politely and turn away, but she found she couldn’t force herself to turn.</p><p class=\"class_sh\">Around his eyes, true-black, smooth skin forced the orbs into starker contrast, making their deep shades seem almost to glow. Subtle hints of grey lines ran under that skin in patterns very like spell-lines but somehow utterly different. Like seeing her own language written with the phonetic alphabet. The concepts seemed familiar while remaining utterly opaque to her interpretation.</p><p class=\"class_sh\">She tried to turn away, again, and actually felt resistance like she was fighting herself. A tingle of her own power, emanating from her keystone, proceeded the answer: <span class=\"class_s5rm\">Allure. He’s somehow manipulating the conceptual nature of reality, forcing my attention to remain locked on him.</span></p><p class=\"class_sh\">As an Immaterial Mage, she could work with non-substance aspects of the world, such as gravity, dimensionality, and molecular cohesion, but warping the magnitude of <span class=\"class_s5rm\">concepts</span>? That… that had disturbing implications.</p><p class=\"class_sh\">As if in response to her thoughts, a different set of lines seemed to flicker into prominence around those wounding eyes, and she found herself turning away in confusion. <span class=\"class_s5rm\">What is wrong with me? I stare at something I’ve never seen before and suddenly insist that it must be magic.</span></p><p class=\"class_sh\">She shook her head at her own foolishness. Then, another prickle rippled out from her keystone, a subtle warning, and she froze. <span class=\"class_s5rm\">Conceptual manipulation… would the concept of believability count?</span> She spun, her eyes ripping across the crowds, trying desperately to find the arcane once more. She had the flickering impression of an amused smile but nothing more.</p><p class=\"class_sh\">After another few moments of frenzied searching, she was left with a subtle, low-level itch from her keystone and the growing <span id=\"page_19\"></span>concern that she’d somehow imagined the brief encounter. <span class=\"class_s5rm\">I… I need to get to the Caravanner’s Guild.</span></p><p class=\"class_sh\">Why had she allowed herself to get lost in her own musings once more?</p><p class=\"class_sh\">Tala huffed. <span class=\"class_s5rm\">I’m never going to get anywhere if I don’t get going.</span></p><p class=\"class_sh\">Without a backward glance, she passed through tremendous gates, the southernmost of eight sets, to breach the gargantuan innermost walls.</p><p class=\"class_sh\">Those walls were also carved with beautiful, intriguing reliefs, showing the Builder’s attention to detail. <span class=\"class_s5rm\">When building a cage, make it a pretty one.</span></p><p class=\"class_sh\">She sighed, pushing those thoughts away, along with her others. <span class=\"class_s5rm\">A cage with doors flung wide hardly counts.</span> At least, that was what she wanted herself to believe; what she needed to believe if she were going to maintain her own sanity. <span class=\"class_s5rm\">Human cities are to keep violence out, not humans in.</span> She did <span class=\"class_s5rm\">not </span>contemplate that the results were virtually indistinguishable.</p><p class=\"class_sh\">She strode purposely onward, now, and though she had to ask for directions twice, it took her less than an hour to find the building that she sought. When she did, she hesitated, standing across the street and observing the flow of traffic in and out of the building, itself.</p><p class=\"class_sh\"><span class=\"class_s5rm\">This is it, Tala. You need to decide. Will you take the easy way? Or risk it all?</span> She laughed. It was hardly a risk. Even if no one would hire her, the academy wanted her to pay them back, plus she had her parents’ debts, which had led to her sale into the academy’s tutelage. No, they wouldn’t let her stay unemployed, though who knew what pittance they’d give her if they were forced to find employment on her behalf…</p><p class=\"class_s5u\">Not helping, Tala.</p><p class=\"class_sh\">She took a deep breath and let it out slowly. <span class=\"class_s5rm\">Now or never.</span></p><p id=\"page_20\" class=\"class_sh\">Without further delay, she strode through the wide, double doors.</p>".to_string(),
        ("b1", 1) =>  "<div class=\"heading_s2u\"><p class=\"class_s2p\">Chapter: 2</p><p class=\"class_s2s\"><span class=\"class_s5s\">The Caravanner’s Guild</span></p></div><p class=\"class_sh\">Tala took a deep breath as her feet carried her through the front door of the Caravanners’ main office.</p><p class=\"class_sh\">The doors were simple, if wide, and they stood open, allowing for easy foot-traffic in and out, of which there was a steady flow. The arch which held the doors was easily wide enough for four people—five of Tala’s size—to come through shoulder to shoulder, with a bit of room to spare.</p><p class=\"class_sh\">The room she entered was a wide receiving hall, with clerks working in alcoves around the outside, as well as some more senior workers moving through the shifting groups of their prospective clients.</p><p class=\"class_sh\">Here, almost every business was represented.</p><p class=\"class_sh\">Restaurants negotiated food shipments either for more specialized crops not grown within this city or beginning to establish contracts for when the city’s farming phase ended; artisans similarly negotiated for materials and to ship their goods to other cities; and countless others sought or negotiated similar services.</p><p class=\"class_sh\">The Caravanners also carried mail from city to city, along with other goods, and they did a brisk trade in that respect.</p><p class=\"class_sh\">In truth, this guild was one of the pillars of human civilization. They were unique in the quantity and regularity of their ventures through the arcanous wilds. Only the Builders dealt with beasts more often than the Caravanners, and they didn’t do trips <span class=\"class_s5rm\">through</span> the wilds so much as they fielded vast, long-<span id=\"page_22\"></span>term expeditions out <span class=\"class_s5rm\">into</span> them, building the continuous wave of cities. Well, there was the Harvesters’ Guild, but their goal was slaying beasts and taking from them, so it was hardly a fair comparison.</p><p class=\"class_sh\">She returned her mind to her present time and place. <span class=\"class_s5rm\">There is power within these walls.</span> She felt a growing sense of excitement at the prospect of working for such an important group.</p><p class=\"class_sh\">She had barely taken five steps through the door before she was noticed by a clerk with copper and silver spell-lines covering her face, clearly focused around her eyes. “You! Mage. Can I help you?”</p><p class=\"class_sh\">Tala smiled and strode over to the young woman, where she waited behind a high counter. The clerk was not wearing Mage’s robes, opting instead for a simple, if elegant, single-piece dress. It allowed her freedom of movement, without being a distraction for those she worked with. She had long, dark-blonde hair, pulled into a loose braid. Tala almost frowned at that. <span class=\"class_s5rm\">I’m seeing a lot of inscribed with hair. Is there something different about the inscribers in this city?</span> Now was hardly the time for that line of thinking, however. Tala smiled. “Yes, I am looking for work.” If Tala had to guess, the clerk was only a few years older than she, herself.</p><p class=\"class_sh\">The woman nodded. “I’d hoped so. May I?” She tapped the scribing around her eyes.</p><p class=\"class_sh\"><span class=\"class_s5rm\">Be decisive. </span>Tala nodded once.</p><p class=\"class_sh\">The clerk blinked, seemingly with specific intent, and her spell-lines pulsed with power.</p><p class=\"class_sh\">As before, Tala’s keystone let her know that she was in close proximity to, or the target of, magic, but the feeling wasn’t unpleasant. <span class=\"class_s5rm\">A simple inspection.</span></p><p class=\"class_sh\"><span class=\"class_s5rm\">As before?</span> She had the stuttering impression of blood and darkness but couldn’t pull a coherent memory together. <span id=\"page_23\"></span><span class=\"class_s5rm\">Must have been a bad dream.</span> She dismissed the fractured recollection without further thought.</p><p class=\"class_sh\">To Tala’s unenhanced eyes, the effect on the clerk’s face looked very similar to a heat haze, though with a little more light to it. Even that indication was a vast improvement on what Tala had seen before her time at the academy. <span class=\"class_s5rm\">My body is acclimating to magic detection.</span></p><p class=\"class_sh\">Her instructors had said that, in time, she wouldn’t need to continue getting inscriptions for the magesight at all. Her body would learn how to see the signs for itself, and her mind would interpret the input in ways that mimicked the spell-line-granted vision.</p><p class=\"class_sh\">It was, in truth, another thing those teachers didn’t truly understand, but they likened it to a skilled merchant learning to know weights and measures without the need of a scale over time. He could simply pick up a sack and know the weight of its contents. No magic involved.</p><p class=\"class_sh\">Tala had always been skeptical, but it seemed she might have been wrong, again. The tell-tale signs <span class=\"class_s5rm\">were</span> there. <span class=\"class_s5rm\">It would be nice to forgo that expense…</span> Magesight was so often used that the inscriptions around a Mage’s eyes were almost always the most often refreshed.</p><p class=\"class_sh\">She was letting her mind wander, again. She focused back on the clerk, just as the woman nodded and blinked again, deactivating her magesight.</p><p class=\"class_sh\">“Yes, you will do nicely, Mage. Indications suggest an intact keystone.” She smiled widely. “You must have had quite the run of bad luck to so completely deplete the rest of your inscriptions; I can’t detect even a single ripple of non-natural magic from anything <span class=\"class_s5rm\">except</span> your keystone.”</p><p class=\"class_sh\">Tala laughed, nervously. “Yeah, well. I’m alive, and here, so…” She smiled, trying to put forward confidence. <span class=\"class_s5rm\">So much for being able to decide whether or not to be considered a Mage…</span><span id=\"page_24\"></span>She hadn’t considered a magesight inspection this early in the process. <span class=\"class_s5rm\">More the fool, me.</span></p><p class=\"class_sh\">The clerk waved a hand. “I don’t need the details. You are an Immaterial Guide, yes?”</p><p class=\"class_sh\">“Yes…” Tala cleared her throat. “I apologize, but I didn’t catch your name.”</p><p class=\"class_sh\">“Oh! How silly of me. You may call me Lyn Clerkson.”</p><p class=\"class_sh\">“Mistress Lyn, a pleasure to meet you. I’m Tala.”</p><p class=\"class_sh\">“Tala…?”</p><p class=\"class_sh\">“No family name.”</p><p class=\"class_sh\">“Mistress Tala, then.” Lyn smiled.</p><p class=\"class_sh\">Tala extended her hand.</p><p class=\"class_sh\">Lyn shook it happily. As she did so, her sleeve pulled up, and Tala was able to get a better look at the extensive spell-lines twining about Lyn’s forearm, wrist, and hand. <span class=\"class_s5rm\">So, a full Mage?</span> Or she was just more heavily inscribed than the non-Mages Tala was used to.</p><p class=\"class_sh\">“Are all the clerks here Mages?”</p><p class=\"class_sh\">“Oh, no. I’m one of the Senior Exchequers, here. Specifically, I’m in charge of the recruiting and handling of new recruits.” She made a motion with her arms that mimed excitement. “Yay! Right? I’m glad I was here when you wandered in.”</p><p class=\"class_sh\">Tala blinked at Lyn several times, trying to figure out what to make of the girl. “Yeah. I suppose I’m glad, too.”</p><p class=\"class_sh\">“So, have you ever empowered bigger boxes?”</p><p class=\"class_sh\">She blinked several times, trying to make sense of the question. “What?”</p><p class=\"class_sh\">“Apologies. That’s how I always think of them. I mean have you ever empowered spatial enlargement scripts? Not many Mages have, outside the Caravanners’ Guild, but I figure it’s good to ask.”</p><p class=\"class_sh\">“Oh! You mean expanding the available space within a given container?”</p><p id=\"page_25\" class=\"class_sh\">Lyn brightened. “Yes! Do you have experience?”</p><p class=\"class_sh\">“Some, but not on any large scale.” The idea had fascinated Tala enough that she’d pestered a teacher into giving her extra lessons and materials on the subject. Even so, she’d only empowered the spell-lines involved a few times.</p><p class=\"class_sh\">Lyn’s smile grew, genuine excitement evident in the expression. “Oh, that’s just wonderful! Teaching new Mages how to twist their mind ‘just so’ can be a… time-consuming process.”</p><p class=\"class_sh\">Tala nodded in acknowledgment. “Yeah, it took me nearly a month before I was able to get past the mental blocks.”</p><p class=\"class_sh\">Lyn laughed, and her tone took on that of someone quoting an oft-heard refrain. “If you don’t believe it’s possible, it isn’t.”</p><p class=\"class_sh\">Tala smiled in return. <span class=\"class_s5rm\">I just might like working with you, Lyn.</span></p><p class=\"class_sh\">“But only a month? That is quite quick!” She paused, then cleared her throat. “You don’t have to answer this, but I have a pet theory I’d like to test.”</p><p class=\"class_sh\">Tala tilted her head, curious herself. “Oh?”</p><p class=\"class_sh\">“Did you have any background in physics or geometry before your first attempt?”</p><p class=\"class_sh\">She laughed. “No! And having spatial distortion theory in my head definitely made those harder to tackle.”</p><p class=\"class_sh\">A small, knowingly contented smile tugged at Lyn’s lips. “I’d thought so! It always seems that the more ignorant Mages are able to master more obscure aspects faster.” She paled, her smile faltering. “I am so sorry! I didn’t mean—”</p><p class=\"class_sh\">Tala held up a hand, grinning. “No harm meant; no harm done. I <span class=\"class_s5rm\">was</span> ignorant.”</p><p class=\"class_sh\">Lyn cleared her throat. “Even so. I apologize.” She took a deep breath and let it out quickly. “Now, then. We really should get to business. Are you looking for work on your way to a particular city, work within this city, or were you hoping for a longer-term contract?”</p><p id=\"page_26\" class=\"class_sh\">Tala’s grin slipped back to a casual smile. Her research had not been in vain. <span class=\"class_s5rm\">Once I’ve enough to fund my own inscriptions, I can just do piecework to get between cities.</span> That would leave her free to do as she pleased… <span class=\"class_s5rm\">Once my debts are paid off…</span> Her smile weakened, just slightly.</p><p class=\"class_sh\">“Longer term is better paid, and we do offer signing bonuses for certain contracts, and an Immaterial Guide with spatial distortion experience is definitely in that wagon!” After a brief pause, she added, “At least for certain contract lengths.”</p><p class=\"class_sh\">“What is the shortest contract with a signing bonus?”</p><p class=\"class_sh\">“Hmmm… Let me see.” She pulled out a stone slate and began manipulating the text on the surface, seemingly flipping through magically stored pages. “It looks like, for your quadrant, we can offer a contract of one year or ten trips, whichever is completed sooner. You are obligated to take a minimum of one trip every other month, including within a week of first signing.”</p><p class=\"class_sh\">“And the rate?”</p><p class=\"class_sh\">“Four ounces per trip, and the signing bonus is four ounces.”</p><p class=\"class_sh\">Tala deflated. One ounce of silver would buy a good meal, but not much more than that. That was lower than an average worker’s day wage, and she doubted the trips only took a day. <span class=\"class_s5rm\">How do people survive on so little? </span>“How often could I take trips? Is there a minimum waiting time?”</p><p class=\"class_sh\">Lyn blinked, seemingly confused at Tala’s dour tone. “No… but even the shortest trips take nearly a week, and most Mages like to have time to spend their earnings in whichever city they arrive in. That, on top of getting re-inscribed and allowing any change to the scribings to set… I’ve known very few to make a trip every month.” She wobbled her head slightly, seeming to hedge. “Well, excepting those who do ‘out and back’ work. Those tend to do two trip blocks, then take longer breaks in between.”</p><p class=\"class_sh\">“Time to spend…” She was frowning.</p><p id=\"page_27\" class=\"class_sh\">Lyn opened her mouth in an understanding ‘Oh!’ “Apologies, again, Mistress Tala. Four ounces <span class=\"class_s5rm\">gold.</span>”</p><p class=\"class_sh\">Tala found herself frozen in surprise. <span class=\"class_s5rm\">Four ounces… gold.</span> An ounce of gold was a hundred times as valuable as one of silver. <span class=\"class_s5rm\">Yeah, a month to relax after each trip would be quite nice.</span> That, and her debt to the academy, on top of her parents’ debt… <span class=\"class_s5rm\">Now, also mine…</span> was 487 ounces gold, twenty ounces silver. <span class=\"class_s5rm\">One hundred</span><span class=\"class_s5rm\">twenty-two trips. Ten years.</span> She’d been expecting the debt to follow her for her entire life unless she found alternate means of paying it off. <span class=\"class_s5rm\">I can make ten years work. </span>Though, she wasn’t accounting for expenses.</p><p class=\"class_sh\">Lyn quirked a questioning smile. “You haven’t done much contract work, have you? You don’t seem to have a good idea of your value.”</p><p class=\"class_sh\">“Clearly not.” No one had been willing to give her solid data.</p><p class=\"class_sh\">“Well, that is our fault. If we advertised better, maybe we’d have gotten you in here sooner!” Her smile firmed up. “And I can assure you, with as well-traveled as you’ll be after even a short contract, we wouldn’t dream of underpaying you. We’d never hold onto Mages if we tried that.” She gave a little chuckle.</p><p class=\"class_sh\">Tala nodded distractedly, not really hearing Lyn’s continued dialogue. “Maybe… Is there a slightly longer contract available? Could I negotiate better rates for two years or twenty trips? A higher signing bonus? Oh! And after the contracted trips, what is the piece job rate, going one way?”</p><p class=\"class_sh\">“All great questions. If you aren’t on a contract, and we have a caravan in need of a Mage of your type, your rate would be three-and-a-half ounces gold, though that can vary slightly from trip to trip. For a three-year, twenty-trip contract, the best I can offer is a trip rate of four-and-a-half ounces, with a one-trip-value signing bonus.”</p><p class=\"class_sh\"><span class=\"class_s5rm\">Ninety-four-and-a-half ounces gold…</span> Tala was speechless. Even with her inscriptions, that should cover over a sixth of her <span id=\"page_28\"></span>debt, with some to spare. She <span class=\"class_s5rm\">thought</span> she had a good guess of how much her spell-lines would cost. She hesitated.</p><p class=\"class_sh\">Lyn’s smile grew. “It won’t increase the signing bonus beyond four-and-a-half ounces, but if you sign a five-year or thirty-trip contract, I can give you five ounces per trip. You won’t be as free to choose your destinations, as those rates are a bit too much except on more lucrative runs.”</p><p class=\"class_sh\">“What about frequency?”</p><p class=\"class_sh\">“There are <span class=\"class_s5rm\">many</span> of those leaving every week, but they tend to be a bit longer, closer to two weeks on average.” She hesitated. “I should be clear, even at the lower rates, the trips will range from one to four weeks. You could always choose the shorter trips, but that is frowned upon—as you can imagine. We try to give as much freedom as possible, but we don’t like to see that abused.”</p><p class=\"class_sh\">Tala nodded. <span class=\"class_s5rm\">Five years.</span> She hesitated. <span class=\"class_s5rm\">No, thirty trips. Each around two weeks…</span> She could fulfill her contract in less than half the prescribed time. <span class=\"class_s5rm\">Just about thirty percent of my debt gone in a year and a half, in one contract? That’s a great start, Tala</span>. She grinned. “I’m interested in a thirty-trip contract, but let’s talk terms. What all is provided on the trips? Do I need to bring my own supplies, shelter, gear? What expenses should I expect to bear, and what ancillary support will the guild be providing?”</p><p class=\"class_sh\">Lyn’s smile turned slightly predatory. “Let’s see what we can work out.”</p><p class=\"class_s5h\">*<span class=\"class_s5rr\"></span>*<span class=\"class_s5rs\"></span>*</p><p class=\"class_sv\">Nearly two hours later, Lyn and Tala sat across from each other in comfortable chairs, sequestered in a back room of the Caravanner’s headquarters.</p><p class=\"class_sh\">Empty mugs of tea stood on the table between them, alongside a contract.</p><p id=\"page_29\" class=\"class_sh\">“Here.” Lyn turned the scripted stone tablet around, passing it back to Tala. “I think this represents everything we’ve agreed to.”</p><p class=\"class_sh\">The text was not written on the stone, though it seemed to be. The words were manifest there from the contract archive, and once Tala willingly put a drop of her blood to the slate, with the intent to confirm the agreement, it would be logged as officially binding. Lyn had already placed her own blood in one corner, using a small, sharp protrusion on the tablet, in place for that purpose.</p><p class=\"class_sh\">Tala scanned the document quickly. It outlined a statement of her own qualifications; those that were verified within the system, such as her certification as a Mage, were highlighted, while those based on her word were set apart. The wording, and the magic in the contract, would annul any obligation from the Caravanner’s Guild if she had been false. Indeed, there were steep penalties if that were to be the case. Thankfully, she’d avoided any falsehoods.</p><p class=\"class_sh\">Beyond her own merit, the agreed-to payments were outlined, along with other restrictions and benefits.</p><p class=\"class_sh\">She was required to have a certain level of preparedness before accepting an assignment, as well as to modify her preparations to meet any specific requirements for the given trip. She would additionally be granted food for the duration of any voyage. She had forgone the standard offerings of an attached servant, to manage the day-to-day responsibilities, and a private wagon for her personal residence while outside city walls.</p><p class=\"class_sh\">Her magics, once she was reinscribed, were mostly bent towards survival, so safety shouldn’t be a concern. As to the convenience of it, she could bear a little discomfort to pay off her debt more quickly.</p><p id=\"page_30\" class=\"class_sh\">That in mind, she’d negotiated for greater pay in exchange for less convenience and a bit more danger.</p><p class=\"class_sh\">Thus, the agreed to per-trip payment, as well as her advance, had been raised to five-and-a-half ounces gold, and she would not be limited to the high-value or longer missions. Apparently, most Mages expected a luxuriously appointed carriage and highly skilled servant, and Tala had gotten Lyn to admit that those items easily cost the guild upwards of one-and-a-half ounces gold per trip. Thus, Tala was offering them a bargain.</p><p class=\"class_sh\">Everything on the contract was, indeed, as they’d agreed, and it was written with plain, easy-to-understand language, as Common Law demanded.</p><p class=\"class_sh\">Tala pricked her finger on the sharp nub, and it retracted immediately after.</p><p class=\"class_sh\">With an effort of will, she allowed her gate to open, and magic flickered through her body, infusing her blood just as she touched the cool stone. The drop of blood that had been building on her finger vanished into the stone, and the tablet turned a pleasant, emerald green, denoting full confirmation.</p><p class=\"class_sh\">Without an inscription to direct and release its power, the magic still flowing through her left Tala with a nervous energy. She wanted to get up and run. Her keystone didn’t help, as it wasn’t meant to use up excess power.</p><p class=\"class_sh\">Lyn had been watching the contract, and when she noted the change to green, she smiled. “Your consent, as well as your words, have been accepted.” She looked up at Tala. “Welcome!” Her smile spread with genuine enthusiasm. “I’m so glad that you came to us.” She tilted her head, seeming to consider for a moment. “Do you have an inscriber in the city, yet?”</p><p class=\"class_sh\">Tala thought about Phoen’s friend, but she didn’t really know them well, so she shook her head. “No.”</p><p class=\"class_sh\">Lyn’s smile seemed to settle into one of satisfaction. “I figured not. Now, no self-respecting inscriber would dare get handsy <span id=\"page_31\"></span>with a Mage of <span class=\"class_s5rm\">our</span> guild, but I know of one who’s better than average.”</p><p class=\"class_sh\">Tala… hadn’t thought of the issue of finding an inscriber herself. She nodded gratefully. “Thank you. Are they your inscriber?”</p><p class=\"class_sh\">“She is, yes.” Lyn nodded. “Though it’s one of her apprentices that does the work on me, directly. She’ll have closed up for the evening, but I know where she likes to grab dinner. We can join her if you’d like, and if you two get on, you can have your spell-lines inscribed tomorrow.”</p><p class=\"class_sh\">Tala’s eyes flicked to Lyn’s hair. Though it was held up in a utilitarian style, it was clearly quite long. Even so, Tala thought she saw hints of spell-lines among the roots, confirming her suspicion that something was different about this city’s inscribers. A smile tugged at her lips. “That sounds like a great plan.” She hesitated, her smile faltering, but after a moment’s indecision, she decided to push forward. “When would I get my advance?”</p><p class=\"class_sh\">Lyn’s smile shifted, again, becoming a knowing smirk. “We can grab it for you on the way out. I’m off anyways.”</p><p class=\"class_sh\">“Oh! I held you up?”</p><p class=\"class_sh\">Lyn waved away the concern. “Not really. I always have to finish up my work, regardless of the time. Today? Getting this contract worked out was the priority.” She stood, smoothing out her simple dress.</p><p class=\"class_sh\">For the most part, Mages’ robes had quick-release ties so that the Mage could shed the garment with speed. Most Mages expressed their power from many locations, so cloth coverings added difficulty and expense when the spells breached the cloth to escape.</p><p class=\"class_sh\">There was also the danger, in more restrictive clothing, that a garment could pull the skin in an unexpected manner, altering a Mage’s spell-lines in unexpected or dangerous ways. The net <span id=\"page_32\"></span>result was that most Mages wore as little as they could manage while casting and covered themselves with Mage’s robes in between such workings.</p><p class=\"class_sh\">Tala… well, she ascribed to a different philosophy of casting. She ensured that the manifestations of all outward expressions of power originated from her hands. It was a weakness if she were ever truly hampered, but she’d seen that as an acceptable tradeoff.</p><p class=\"class_sh\">Lyn’s own choice of a simple dress spoke volumes about her life, as well as her work as a Mage. She did not expect, or have need, for quick, complicated castings, nor did she seem to have any concern about having to remain mobile. In short, she led a safe life.</p><p class=\"class_sh\">“Tala?”</p><p class=\"class_sh\">“Hmm?”</p><p class=\"class_sh\">Lyn was standing, half turned away, seeming to be waiting. “Are you coming?”</p><p class=\"class_sh\">“Oh!” Tala stood in a rush. She’d allowed her mind to wander, again. “Yes. Let’s go.”</p><p class=\"class_sh\">Tala followed as Lyn led her through the now mostly empty main hall of the guild. They came to a small counter, tucked into a back corner, where an unlined clerk asked Tala for a drop of blood.</p><p class=\"class_sh\">The clerk confirmed her contract and that money was owed. He frowned when he saw the amount, and Lyn was forced to take him aside for a quick, quiet conversation. Apparently, no one had received a signing bonus as high as Tala’s during his time working this station.</p><p class=\"class_sh\">Finally, he was satisfied, and he presented Tala with a small pouch of coins. She counted it, at his prompting, and when she had verified the amount, he marked her as having been paid. That complete, he hesitated. “I know it isn’t my place, but may I offer a word of advice?”</p><p id=\"page_33\" class=\"class_sh\">Tala had already begun to turn away but hesitated at his question. “Umm… sure? I’m happy to learn, where I can.” As she responded, she’d turned back towards the middle-aged man.</p><p class=\"class_sh\">“Always count your pay. No one <span class=\"class_s5rm\">should</span> ever try to short you, but mistakes happen, and after you confirm receipt, even the best-intentioned pay clerks can’t give you more.”</p><p class=\"class_sh\">She contemplated that for a long moment, then nodded. “I see.”</p><p class=\"class_sh\">He quirked a smile. “If anyone gives you grief for counting, it is reasonable for you to remind them that you are giving your word that you received the full amount. The only honorable thing for you to do is check before so swearing.”</p><p class=\"class_sh\">She smiled in turn. “Clever. I’ll remember that. Thank you.”</p><p class=\"class_sh\">He gave a small bow. “Welcome to the guild, Mistress Tala.”</p><p class=\"class_sh\">She gave a nod in return. “Thank you.” She hesitated. “I’m sorry, I didn’t catch your name.”</p><p class=\"class_sh\">He blinked at her a few times, then looked down at his tunic.</p><p class=\"class_sh\">Tala followed his gaze, then flushed. A small wooden placard was affixed on the left side of his tunic’s chest, his name clearly written out in white lettering.</p><p class=\"class_sh\">He cleared his throat. “You can call me Gram.”</p><p class=\"class_sh\">“Gram… A pleasure to meet you.”</p><p class=\"class_sh\">He quirked another smile. “And you, Mistress Tala.”</p><p class=\"class_sh\">Lyn let out a small laugh, leading Tala away, across the hall, and out the doors.</p>".to_string(),
        _ =>  "<p>Default chapter content.</p>".to_string(),        
    };

    Ok(chapter)
}

#[cfg(not(feature = "mock"))]
pub async fn fetch_cursor_text(book_id: &str) -> Result<CursorTextResponse, String> {
    let url = format!("/api/v1/cursors/{}/text", book_id);

    let resp = get_with_auth(&url)
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to fetch cursor text for {}: {}", book_id, resp.status()).into());
    }

    let cursor_text: CursorTextResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(cursor_text)
}

#[cfg(feature = "mock")]
pub async fn fetch_cursor_text(book_id: &str) -> Result<CursorTextResponse, String> {
    use serde_json::json;
    use serde_json::Value;

    // Define the JSON data per book_id
    let json_data: Value = match book_id {
        "b1" => json!({
            "cursor": {
                "user_id": "pete",
                "book_id": "b1",
                "cursor": { "chapter": 1, "chunk": 0 }
            },
            "text": ""
        }),
        "b2" => json!({
            "cursor": {
                "user_id": "pete",
                "book_id": "b2",
                "cursor": { "chapter": 1, "chunk": 0 }
            },
            "text": "<p>Sample text for chunk 0 of chapter 1.</p>"
        }),
        _ => json!({
            "cursor": {
                "user_id": "pete",
                "book_id": book_id,
                "cursor": { "chapter": 0, "chunk": 0 }
            },
            "text": "<p>Default cursor text.</p>"
        }),
    };

    // Deserialize JSON into your CursorTextResponse
    let cursor_text: CursorTextResponse = serde_json::from_value(json_data)
        .map_err(|e| format!("Failed to deserialize mock JSON: {}", e))?;

    Ok(cursor_text)
}

#[cfg(not(feature = "mock"))]
pub async fn fetch_book_css(book_id: &str) -> Result<String, String> {
    let url = format!("/api/v1/books/{}/css", book_id);
    let resp = get_with_auth(&url)
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Failed to fetch CSS for book {}: {}", book_id, resp.status()));
    }
    let css_text = resp.text().await.map_err(|e| e.to_string())?;
    Ok(css_text)
}

#[cfg(feature = "mock")]
pub async fn fetch_book_css(book_id:&str)->Result<String,String>{
  Ok(".calibre {
  display: block;
  font-size: 1em;
  line-height: 1.2;
  padding-left: 0;
  padding-right: 0;
  margin: 0 5pt;
}
.calibre1 {
  display: table-column-group;
}
.calibre2 {
  display: table-row;
  vertical-align: inherit;
}
.class {
  display: block;
  font-size: 1em;
  line-height: 1.2;
  padding-left: 0;
  padding-right: 0;
  text-align: center;
  margin: 0 5pt;
}
.class_s {
  display: block;
  font-size: 1.125em;
  line-height: 1.2;
  page-break-after: avoid;
  page-break-inside: avoid;
  text-align: center;
  margin: 0 0 0.4736em;
}
.class_s1 {
  color: #0563c1;
  display: block;
  margin: 0;
}
.class_s2f {
  color: #0563c1;
  display: block;
  margin: 0 0 3.29707em;
}
.class_s2p {
  display: block;
  font-size: 0.81818em;
  line-height: 1.2;
  margin: 0;
}
.class_s2s {
  display: block;
  line-height: 1.2;
  margin: 0.719795em 0 0;
}
.class_s2w {
  display: block;
  text-indent: 3.75%;
  margin: 0;
}
.class_s2y {
  border-collapse: collapse;
  border-spacing: 2px;
  display: table;
  margin-bottom: 0;
  margin-top: 0.7125em;
  max-width: 100%;
  text-indent: 0;
  border: gray outset 1px;
}
.class_s2y1 {
  border-bottom-style: solid;
  border-bottom-width: 0.75pt;
  border-left-style: solid;
  border-left-width: 0.75pt;
  border-right-style: solid;
  border-right-width: 0.75pt;
  border-top-style: solid;
  border-top-width: 0.75pt;
  display: table-cell;
  text-align: inherit;
  vertical-align: middle;
  padding: 0.031667em 0.102%;
}
.class_s5h {
  display: block;
  text-align: center;
  text-indent: 0;
  margin: 0.7125em 0 0;
}
.class_s5k {
  display: block;
  text-indent: 3.75%;
  margin: 0.7125em 0 0;
}
.class_s5mr {
  display: block;
  text-indent: 3.438%;
  margin: 0.7125em 0 0;
}
.class_s5mt {
  display: block;
  text-indent: 3.438%;
  margin: 0;
}
.class_s5rm {
  font-style: italic;
}
.class_s5rn {
  text-decoration: underline;
}
.class_s5rn1 {
  color: #0563c1;
  text-decoration: underline;
}
.class_s5rr {
  padding-left: 26.19pt;
}
.class_s5rs {
  padding-left: 30pt;
}
.class_s5s {
  font-size: 0.81818em;
  line-height: 1.2;
}
.class_s5u {
  display: block;
  font-style: italic;
  margin: 0;
}
.class_sf {
  display: block;
  font-size: 1.375em;
  line-height: 1.2;
  margin: 1.6255em 0 0;
}
.class_sh {
  display: block;
  margin: 0;
}
.class_sk {
  border-bottom-style: solid;
  border-bottom-width: 0.75pt;
  border-left-style: solid;
  border-left-width: 0.75pt;
  border-right-style: solid;
  border-right-width: 0.75pt;
  border-top-style: solid;
  border-top-width: 0.75pt;
  display: table-cell;
  text-align: inherit;
  vertical-align: middle;
  padding: 0.031667em 0.099%;
}
.class_skc {
  border-collapse: collapse;
  border-spacing: 2px;
  display: table;
  margin-bottom: 0;
  margin-top: 0.7125em;
  max-width: 100%;
  min-width: 100%;
  text-indent: 0;
  width: 100%;
  border: gray outset 1px;
}
.class_st {
  display: block;
  margin: 2.3712em 0 0;
}
.class_sv {
  display: block;
  margin: 0.7125em 0 0;
}
.class1 {
  display: block;
  font-size: 1em;
  line-height: 1.2;
  padding-left: 0;
  padding-right: 0;
  text-indent: 3.75%;
  margin: 0 5pt;
}
.class2 {
  display: table-column;
  width: 98.66%;
}
.class3 {
  display: table-row-group;
  text-align: left;
  text-indent: 1.2em;
  vertical-align: middle;
}
.class4 {
  display: table-column;
  width: 100%;
}
.heading_s2u {
  display: block;
  font-size: 1.375em;
  line-height: 1.2;
  margin-bottom: 1.6255em;
  margin-top: 1.6255em;
  page-break-after: avoid;
  page-break-inside: avoid;
  text-align: center;
  text-indent: 0;
}
.heading_s5mf {
  display: block;
  font-size: 1.125em;
  font-weight: normal;
  line-height: 1.2;
  page-break-after: avoid;
  page-break-inside: avoid;
  text-align: center;
  margin: 1.62598em 0;
}

@page {
  margin-bottom: 5pt;
  margin-top: 5pt;
}".to_string())
}