
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PrettySpineItem {
    pub index: usize,
    pub number: usize,
    pub title: String,
}




use crate::infra::auth::get_with_auth;
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
    if(book_id=="b2"){
      return Err("b2 tests failure".to_string())
    }

    let chapter = match (book_id, chapter_index) {
        ("b1", 0) => "<h1 class=\"chapter-no\" id=\"ch-1\"><a id=\"page_1\">CHAPTER 1</a></h1><a id=\"page_1\"><h1 class=\"chapter-title\">IA</h1><p class=\"para\">SHE WAS SEVENTEEN, and her life was about to end.</p><p class=\"indent-para\">The girl pushed back strands of her tangled black hair and pressed her forehead flush against the glass window. Outside were twenty starjets from the Royal Star Force, the military fleet of the Olympus Commonwealth. A shimmer of panicked whispers flittered around her, and she realized the other passengers had seen the squadron coming their way, too.</p><p class=\"indent-para\">The girl glanced over her shoulder and caught sight of the Elder moving through the crowd of Tawny refugees, trying to quell the growing panic.</p><p class=\"indent-para\">“Why the mif is the RSF here?” she called out to him.</p><p class=\"indent-para\">The Elder shook his head, his lips drawn into a tight line. “If Deus is on our side today, Girl, they’ll wave us by.”</p></a><p class=\"indent-para\"><a id=\"page_1\">The passengers on the ship called her “Girl,” which was fine by her. After traveling with them for a month, she didn’t know their real names either. Her dark hair and golden skin stood out in this crowd like a crooked screw on a brand-new sheet of </a><a id=\"page_2\">metal. The twenty-seven Tawny refugees on the ship all had milky complexions and hair as blue as the deepest ocean. That was what happened in the All Black. You got mixed up with all sorts of different people just trying to survive.</a></p><a id=\"page_2\"><p class=\"indent-para\">The RSF starjets knocked into the transport ship’s hull, and the ship pitched forward. The girl heard the screech of metal scraping against metal, of screws twisting and connecting. The RSF had sealed its docking bridge to the transport ship’s air lock.</p><p class=\"indent-para\">The Elder held a hand up, attempting to ease the wave of anxiety passing through the crowd. “The air-lock door will hold. They won’t be able to board without permission.”</p><p class=\"indent-para\">The girl held her breath, praying he was right, wishing that door was made from the strongest vinnidium steel instead of salvaged antique metal.</p><p class=\"indent-para\">A tinny voice came through the speakers. “The Royal Star Force requests entry to your vessel.”</p><p class=\"indent-para\">Fear ripped through her. What were they searching for? Weapons? Contraband? She swallowed, her throat tight. The Royal Star Force had a reputation of being rougher out in the Fringe territories. Olympus had no problem taking everything from the many people of the Fringe—their homes, their water, their fuel, and their planets.</p><p class=\"indent-para\">The Elder rushed to an intercom speaker and pressed a thumb to a red sensor. “We are outside Commonwealth territory. This ship is not subject to your jurisdiction. Be kind and pass.”</p><p class=\"indent-para\">But there was no response. Only a gentle hiss permeating through the small holes of the intercom speakers.</p></a><p class=\"indent-para\"><a id=\"page_2\"></a><a id=\"page_3\">Silence settled, and then came light taps on the metal, one for each hinge on the entry door. From all her years traveling in the All Black, surviving scuffles against pirates and black-market traders, the girl knew what that sound meant.</a></p><a id=\"page_3\"><p class=\"indent-para\">“Take cover!” she screamed.</p><p class=\"indent-para\">A dull tone reverberated from outside, followed by a loud boom. The doorway flew inward, carving a hole in their ship. Passengers scattered as twisted pieces of metal flew their way. Even with a veil of smoke hanging over them, the girl heard the shuffle of footsteps and armor, and she knew. The Royal Star Force was boarding.</p><p class=\"indent-para\">The Elder grabbed her shoulder. At such a close distance, she saw the gray hairs speckled throughout his navy-blue mane and the age spots dappling his cheeks. But despite his years, his eyes were bright as starlight.</p><p class=\"indent-para\">“Best to stay out of sight.” The Elder pulled her away from the windows, and she was swallowed up by the crowd. Around her, everyone’s shouts merged into one, and she felt her lungs compress as bodies drew tightly together. The smoke settled, and small orbs fixed with camera lenses flew out from the docking tunnel. It was the Commonwealth’s media; they had sent their little Eyes to film and broadcast this entire thing. One camera drone buzzed by so close that it tousled her hair. She glanced at each Eye, watching them change position to find the perfect angle to capture the passengers’ distress.</p><p class=\"indent-para\">It was then she realized: this wasn’t a routine search.</p></a><p class=\"indent-para\"><a id=\"page_3\">“Where is he?” a thick voice bellowed from the other side of the threshold. A formation of officers emerged from the docking bridge, followed by a large figure. The girl recognized </a><a id=\"page_4\">him from the broadcast streams: General Adams, a war hero celebrated throughout the Commonwealth. The medals on his chest—shaped like golden stars and silver olive branches—jingled as he walked.</a></p><a id=\"page_4\"><p class=\"indent-para\">“Where is I. A. Cōcha?” he growled.</p><p class=\"indent-para\">As if on cue, Commonwealth holoscreens appeared around them. On the screens was a Wanted banner, one she had seen over and over again, plastered on interstellar gates, projected on travel hubs—anywhere people could see. The banner had an image of a helmet, a red feather painted across its helm like a stain of blood.</p><p class=\"indent-para\">General Adams was after I. A. Cōcha, a monster with many names: the Sovereign of Dead Space, the Rogue of the Fringe Planets, the Blood Wolf of the Skies. Cōcha was the most dangerous criminal in Commonwealth history.</p><p class=\"indent-para\">General Adams surveyed the faces of the Tawny refugees. “I don’t care much for guessing games. I know he’s here. Send him forward.”</p><p class=\"indent-para\">The girl stayed hidden, waiting for someone to speak. The tension among her fellow travelers thickened, but even with the surmounting pressure, they nodded knowingly to one another, a secret agreement to stay silent.</p><p class=\"indent-para\">“So be it.” Adams motioned for his soldiers to come forward. All fifty of them were armed with laser pistols lethal enough to burn everyone on the ship to ash. The general’s blue eyes glinted like a newly polished dagger. “I’ll just have to shoot all of you.”</p></a><p class=\"indent-para\"><a id=\"page_4\">General Adams raised his weapon at the nearest Tawny, a teenage boy. Adams would kill him, just because he could. </a><a id=\"page_5\">The girl’s heart raced, rage boiling inside her. Surging forward, she grabbed an orb from her side pack and threw it down. A translucent-green energy force field spidered upward to the ceiling, creating a protective wall between the passengers and the RSF.</a></p><a id=\"page_5\"><p class=\"indent-para\">Across the barrier, General Adams grimaced. “Who are you?”</p><p class=\"indent-para\">The girl slammed her palm against a button on the collar of her suit. Her helmet slid on, smooth and automatic. Upon its brow was a blood feather, shining in the darkness. To many, it instilled fear, but to her, it inspired hope.</p><p class=\"indent-para\">The general’s eyes widened in recognition. “I. A. Cōcha.”</p><p class=\"indent-para\">Ia stared him down. “It’s pronounced <em>Eye-yah</em>. You don’t spell my name; you say it.”</p><p class=\"indent-para\">Gunfire erupted, and the air was filled with a flurry of bright-blue energy blasts. They showered around her, absorbing right into the protective wall of the force field.</p><p class=\"indent-para\">This type of force field was called a Carpion shield, designed to block any bullets coming from the other side of the protective wall, but any gunfire originating from within would pierce through. Ia grabbed the energy pistol holstered in her boot and aimed. Shot by shot, her bullets soared through the shield and toward the soldiers. Her aim was precise and clean. But there were too many of them, and she didn’t have enough of a charge to pick them all off.</p><p class=\"indent-para\">She checked the orb at the base of the shield. The meter showed 50 percent strength. The shield was strong, but it wouldn’t last. If she wanted to survive this, she had to act quickly. Ia ran to the overheard bin above her seat and grabbed her pack.</p></a><p class=\"indent-para\"><a id=\"page_5\"></a><a id=\"page_6\">She turned back to the Tawnies. “You have a choice,” she told them. “You can get to the escape pods, or—”</a></p><a id=\"page_6\"><p class=\"indent-para\">She threw her pack onto the floor, revealing her stash of weapons. No more shields, but she did have more than enough guns.</p><p class=\"indent-para\">“This is our ship,” the Elder said, stepping forward. “We fight.”</p><p class=\"indent-para\">Ia pulled out her favorite hand cannons and tossed them to him.</p><p class=\"indent-para\">“Aim for the quartered shield.” It was the symbol of Olympus, a red-and-white shield embroidered on every RSF uniform, stitched on the chest pocket right at their hearts. The perfect target.</p><p class=\"indent-para\">She quickly distributed the rest of pistols to the others in the group, and soon they were firing across the force field.</p><p class=\"indent-para\">The Tawnies had terrible aim. Their rounds burst the pipes, dinged the softer metal of the ship, and hit everything but the RSF officers. The Elder and his group weren’t warriors. They were civilians.</p><p class=\"indent-para\">Ia glanced again at the meter on the Carpion orb. It was at 35 percent. At the state they were in now, they weren’t going to win. She had to call in an even bigger gun for that.</p></a><p class=\"indent-para\"><a id=\"page_6\">She blinked inside her helmet, accessing the ArcLite, a communications system that spanned the known galaxies. A holo-image flickered onto a small panel on the right side of her visor. She sighed in relief at the sight of Einn Galatin’s face. It was like hers. Black hair, golden skin. But her brother’s cheekbones were more pronounced. Sharper, more angular. And his eyes were a different color, a stormy gray instead of </a><a id=\"page_7\">her coal black. An image of two white hearts cast side by side was pinned prominently onto his collar. Their father always told them it was their family symbol. It meant loyalty, a word wasted on the father who had abandoned them long ago, but very fitting for her brother. Einn was the only person she could ever count on.</a></p><a id=\"page_7\"><p class=\"indent-para\">“Where the mif have you been, Ia?” Her brother crinkled his forehead. He did that when he was angry.</p><p class=\"indent-para\">It had been months since she’d seen her brother. With the end of the Uranium War, the Commonwealth’s leaders had increased their efforts at hunting down the criminals on their Most Wanted list. Ia had gone into hiding, hoping the heat on her would eventually die down. It never had.</p><p class=\"indent-para\">“I’m in some deep mung, Einn. The Bugs found me.”</p><p class=\"indent-para\">That was what everyone on this side of the galaxy called the officers of the Star Force: Bugs. Ia had spent her whole life swatting them down, but no matter how many she killed, there were always more who took their place.</p><p class=\"indent-para\">“What?” Einn asked. “How?”</p><p class=\"indent-para\">“Don’t know. I cloaked my signal with Alary tech, but that miffing general still knew I was here.” She grunted, firing another round across the force field.</p><p class=\"indent-para\">Her brother’s eyes darkened. “Get yourself out of there.”</p><p class=\"indent-para\">She shook her head, beads of sweat dripping down her forehead. “It’s not that easy. There are innocents onboard. Tawnies.” She kept her eye on the firefight, watching the Tawnies as they attempted to defend themselves. Their firepower and skill weren’t even close to being enough. “We need your help.”</p></a><p class=\"indent-para\"><a id=\"page_7\"></a><a id=\"page_8\"> “You don’t have to save every refugee who crosses your path.”</a></p><a id=\"page_8\"><p class=\"indent-para\">“Einn,” she whispered. “Please.”</p><p class=\"indent-para\">Her brother shook his head in resignation. “Ping me your location. Try to hold them off until I get there.”</p><p class=\"indent-para\">Her heart leaped. “I owe you one, Brother.”</p><p class=\"indent-para\">“Just survive. That’s all you need to do.”</p><p class=\"indent-para\">Ia nodded. “May your eyes be open, Einn.”</p><p class=\"indent-para\">Her brother said the words as though they had been programmed into his heart. “And your path be clear.”</p><p class=\"indent-para\">It was their farewell, the lines they spoke to each other before they parted at each mission. A secret prayer to keep them safe.</p><p class=\"indent-para\">As he signed off, Ia reminded herself that he was right. She had to get through this. She would find a way. They didn’t call her the Blood Wolf of the Skies for nothing. With their viselike jaws and mighty wings, Lavisian blood wolves were vicious contenders in the Dead Space betting pits. And just like those fierce creatures, if anyone backed Ia into a corner, she was going to bite right down to the bone.</p><p class=\"indent-para\">Ia took aim, gunning down as many RSF soldiers as she could.</p><p class=\"indent-para\">One down.</p><p class=\"indent-para\">Then another. And another.</p><p class=\"indent-para\">Ia smashed her palm against the butt of her pistol, but her ammunitions chamber hummed to silence. She was about to ask someone to toss her another gun when she realized the gunfire on her side of the ship had gone quiet. Their ammo had run out. They had nothing else to use to defend themselves.</p></a><p class=\"indent-para\"><a id=\"page_8\"></a><a id=\"page_9\">She breathed heavily, her eyes shifting from the never-ending fire coming from the Star Force’s side. Each bullet further drained the strength of her shield.</a></p><a id=\"page_9\"><p class=\"indent-para\">The media’s Eyes flew to the front lines, pointing straight at her.</p><p class=\"indent-para\">“It’s over, Cōcha,” General Adams hissed. His white teeth reflected the lights of the cameras. “I don’t even need to wait until that flimsy shield of yours runs out. I can just gas this entire ship and end it now.”</p><p class=\"indent-para\">Ia’s heart pounded deep inside her chest. She glanced back at the Tawnies. This was a passenger ship, which meant there would be only one grav suit, maybe two. They’d never withstand a chemical attack.</p><p class=\"indent-para\">“I see you figured it out.” The general’s voice interrupted the zigzag of her thoughts. “You have enough air in your helmet for what? Two hours? All these Tawnies will be long dead by then.” His quiet calm slashed like a razor into her skin. “Or you can surrender.”</p><p class=\"indent-para\">Ia gazed out the window. Five more RSF battleships had joined the others, completely surrounding the Tawny ship. Each one of them was big enough to house fifty starjets. Even if Einn was on the way, he wouldn’t be able to break through them.</p><p class=\"indent-para\">General Adams turned to one of his lieutenants. “Get the gas ready.”</p><p class=\"indent-para\">Ia heard cries of panic from behind. She glanced back, her eyes landing on a Tawny woman holding her child and shielding his eyes so he wouldn’t have to see their fate. The Elder stroked her hand, trying to keep her calm.</p></a><p class=\"indent-para\"><a id=\"page_9\">Ia took a deep breath as a decision shook her bones. General </a><a id=\"page_10\">Adams might have won, but there was still something she could do.</a></p><a id=\"page_10\"><p class=\"indent-para\">“I’ll surrender. On one condition…” Ia said.</p><p class=\"indent-para\">“Name it.”</p><p class=\"indent-para\">She dropped her pistol, nozzle clanging sharply on the floor. “Take me, but only me.”</p><p class=\"indent-para\">A smile slithered onto General Adams’s lips. “Done.”</p><p class=\"indent-para\">Ia ripped her helmet off, her eyes searing into the general. “We have a deal.”</p><p class=\"indent-para\">The general looked back to the one of the officers. “Tell the ships to clear a path.”</p><p class=\"indent-para\">The Elder looked over at her in alarm. “What are you doing, Ia?”</p><p class=\"indent-para\">“You helped me,” she whispered, just loud enough that the Elder could hear her. “You didn’t have to, but you did.” She glanced at all the Tawny refugees. There were twenty-seven of them, enough to fit into the two escape pods built into the ship. “Get to the pods. My brother will find you.” She looked back to the Elder, and she paused, her heart heavy with guilt. “I should have told you who I am.”</p><p class=\"indent-para\">“We knew, Girl. You can’t outsmart a Tawny.”</p><p class=\"indent-para\">“Then why take me in?”</p><p class=\"indent-para\">His eyes shone at her. “Not all the stories of I. A. Cōcha are bad ones.”</p><p class=\"indent-para\">All this time, they’d known who she was, and they regarded her the way she always wanted to be seen. Not as a monster, but as a person, just like any other.</p></a><p class=\"indent-para\"><a id=\"page_10\">She fought back tears. “Thank you,” she said. Then she jutted out her chin, telling them to go. The shield wall blocked </a><a id=\"page_11\">the Star Force from the starboard side of the ship where the escape pods were located. In her head, Ia counted to thirty, giving the Tawnies enough time to get to the pods.</a></p><a id=\"page_11\"><p class=\"indent-para\">At the end of her count, she put her pistol on the floor, and with her foot, she tapped the Carpion orb. The shield flashed green as it deactivated. The soldiers punched through the fading sheen of the force field and surrounded her, their pistols pointed at her head.</p><p class=\"indent-para\">As she raised her hands in surrender, she felt a brush of air as cameras whizzed around to film her at all angles. One of them stopped, hovering in front of her, blasting its bright white light into her eyes. She squinted at the lens.</p><p class=\"indent-para\">After today, everyone in the known universe would know Ia’s face. And no matter where she went, she would no longer be safe.</p><p class=\"indent-para\">Mif. This was going to suck.</p><p class=\"indent-para\">The soldiers grabbed her arms and bound them behind her back. As she struggled against the binds, she turned to the windows. One of the escape pods had cleared the blockade as promised, its silhouette now a mere speck in the distance. Ia sighed in relief.</p><p class=\"indent-para\">But just as the second pod was about to pass, a RSF battleship closed in, blocking the pod’s escape. Her gut twisted. She had been a fool to think the general would keep his side of the bargain.</p><p class=\"indent-para\">“You agreed to let them go,” she screamed, lunging toward the general. Before she could dig her shoulder into his chest, someone kicked the back of her thighs, forcing her to kneel.</p></a><p class=\"indent-para\"><a id=\"page_11\">A young soldier approached the general, whispering low, </a><a id=\"page_12\">yet loud enough for her to hear. “What about the other escape pod, sir. Should we pursue?”</a></p><a id=\"page_12\"><p class=\"indent-para\">“Don’t waste your time,” General Adams told the soldier and then nodded over at Ia. “She’s the one we want.”</p><p class=\"indent-para\">Still seething, Ia whispered a silent plea, praying Einn would find the first escape pod. He would see that those Tawnies were safe. And after that, he would come to rescue her. Guns in both hands, he would board the ship they were in and shoot the general right between the eyes.</p><p class=\"indent-para\">The general noticed her glare. He crouched to face her. His rough fingers gripped her chin so she couldn’t look away. “You lost, Cōcha. What do you have to say for yourself?”</p><p class=\"indent-para\">If she couldn’t escape and she couldn’t kill him, she decided to do the next best thing. Like a viper, she sprung her head forward, her aim sharp and certain. Her forehead cracked the general hard in the nose.</p><p class=\"indent-para\">Seconds later, the hard grip of a pistol smacked her in the back of the head. She fell forward, catching a wonderful glimpse of blood dripping down the general’s lips and chin. Before she slipped into unconsciousness, she smiled.</p><p class=\"indent-para\">If she was going down, she was going to do it one way and one way alone.</p><p class=\"indent-para\">Gloriously.</p></a>".to_string(),
        ("b1", 1) =>  "<h1 class=\"calibre1\">Chapter 61 - Fight-View Restaurant</h1><div class=\"calibre2\"><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala, Rane, Lea, and Terry slowly rode the clean, well-maintained elevator up toward the top floor of the fight-view restaurant.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">They hadn’t had any trouble gaining entry into the city—not only were they both still well known, Master Grediv had left word of their imminent arrival—and the setting up of Ironhold’s gate at the usual spot had gone off without a hitch. That done, they’d had no reason nor desire to delay their meal with Master Grediv and his introduction to Lea.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Terry rode on Lea’s shoulder, perched happily atop her new dress, which the girl had asked Tala to make for her. It was similar to a sundress, but a bit longer and made of slightly heavier fabric.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">It was a slightly pastel ruby red, the subdued color setting off Lea’s clearly magical eyes all the more for the purity of their shade.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The girl had put her hair into a braid, mimicking her mother’s style, and both her hands held the braid as if it were a lifeline.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane rested his hand on her shoulder, the one Terry wasn’t occupying. “You’ll be fine. Master Grediv is a kind man.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“But you still call him ‘Master’… Is he that formal?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane smiled. “No. It is more something that I choose to do than something he insists upon. You are welcome to ask him. The worst that he will say is to confirm your use of the moniker.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea gave a slow nod. “Alright.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala gave her daughter a reassuring smile. “He’s powerful, but your father and I are his match in advancement. This is his city, but there is no way he’ll be able to do anything to you with us around.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea blanched. “Do to me? He might do something to me?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala hesitated. “Oh… that wasn’t your concern?” Rane was giving her a rather intense ‘are you kidding me?’ stare, and Tala cleared her throat, continuing in a placating rush. “Of course that wasn’t. How silly of me to even mention it.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea seemed a bit unsteady as the lift came to a stop, the doors opening to let them out into the vaulted, gorgeously appointed space.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Terry headbutted Lea’s cheek and cooed softly, for once seeming to consider their surroundings as he chose his volume of communication.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea relaxed a bit, releasing her braid with one hand to scratch Terry’s head. “Thank you, Terry.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Terry trilled softly in reply, nuzzling into her hand.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The top floor was just as busy as the last time that Tala had visited, with magical privacy bubbles keeping the noise to a minimum as servers and staff hurried to and fro.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The difference from Brand’s establishment was stark, though the various Archive slates displaying fights—along with the obvious food—tied the two together in theme and general purpose.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“It’s so quiet.” Lea practically whispered.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane leaned in a bit. “Can you see the magic around each table?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea squinted. “I can see… something? I don’t know what it means, though.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane nodded. “You are still learning how to identify what you see. Those are privacy bubbles, areas in which magic is keeping sound from coming </span><em class=\"calibre4\"><span class=\"calibre3\">out</span></em><span class=\"calibre3\">.” He emphasized that last word. “If we are loud, they will hear us just fine.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">She nodded, seemingly unwilling to speak even quietly now that she fully understood.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala leaned close, speaking softly, but pointedly not whispering. “It’s okay to talk; we just need to be polite about it.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea nodded again. “Okay, Mom.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">At the central table, right near the massive transparent wall, Master Grediv waited for them.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Upon their entry onto this level of the restaurant from the elevator, he stood and smiled, eyes tracking over each of them before returning to regard Lea more closely.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The group crossed the floor, stopping just inside the range of the Stone Holder’s table’s privacy bubble. He activated that, and they exchanged appropriate bows. Tala decided to let Rane address his old master first. “Master Grediv. It is a pleasure to see you.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv smiled. “Rane, the pleasure is mine. Mistress Tala, Terry, I am glad to see you two, as always.” He then turned to Lea, bowing again, far more deeply than he should have given their relative advancement. “Young Lea. I welcome you as a member of my family, however far removed.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea was practically bouncing as she bowed in return. “Greetings honored ancestor. I am glad to meet you.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">He gestured for them to sit, and as they did so, Lea got a contemplative look on her face.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“Do we call you ‘honored ancestor’ because ‘honored’ is the arcane equivalent of your level of advancement?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The Paragon tilted his head to the side in thought before shrugging. “I’m not sure. That might have been the origin, but I don’t believe that those of other advancements are addressed differently by their descendents.” He then glanced to Tala and Rane. “You have decided to discuss advancement with her?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">His tone made his surprise evident. Tala shook her head. “Nothing specific. We are keeping to that tradition, but she could hardly be unaware of the differences between Mages of various levels.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv gave a slow nod. “And with the Eskau in your hold, she has been exposed to arcanes and their advancement, at least in general.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane smiled. “Yes, we did give her the basic instruction there before we allowed Eskau Meallain to meet her.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea huffed. “Not that it mattered in the end.” She looked to Master Grediv, as if hoping to find someone to be on her side. “She got angry that I was me, was placated, then fought mom. It was all a bit silly.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane barked a laugh, even while Tala tried to stifle her own.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv arched an eyebrow. “Oh? She attacked your mom?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea shook her head in negation. “No, no. It was a spar. It seemed like she wanted that more than to meet me.” There was </span><em class=\"calibre4\"><span class=\"calibre3\">something </span></em><span class=\"calibre3\">in the girl’s tone, but it wasn’t quite sadness or disappointment. “Still, their fight was interesting.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“I see.” He looked to Rane and Tala once more. “I know that she has been examined by many people, but would you permit me a quick magical scan?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">They shared a look before Rane shrugged. “</span><em class=\"calibre4\"><span class=\"calibre3\">We</span></em><span class=\"calibre3\"> have no issue with it.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">He left the implication hanging, and Master Grediv turned to Lea. “Lea, may I use some magic to quickly scan you?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea looked to her parents, and when they both smiled encouragingly, she agreed. “Yes. I think that would be okay.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“Thank you, child.” Master Grediv pulsed slightly to Tala’s threefold perception, his magic reaching out and passing through Lea. The area around them seemed to echo his power—his authority—and then it was done, the working having passed in less than a heartbeat’s time.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">He leaned back, clearly contemplating.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane noticed his old master’s demeanor, seemingly realizing that the man would be lost in thought for a bit. “I’ll go order some food.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala smiled. “Thank you, Rane.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">He returned the smile. “Of course.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea perked up. “Do they serve ribs? Mistress Petra made salt and pepper short ribs the other night, and they were </span><em class=\"calibre4\"><span class=\"calibre3\">fantastic.</span></em><span class=\"calibre3\"> I want to try more types.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane’s smile shifted to a grin. “I’ll see what they have.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“Thank you, dad!” She was obviously gleeful at his agreement.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">He leaned over and kissed the top of her head. “Of course.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Without another word, he went in search of a server.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Terry and Lea played a bit, with the terror bird flickering from shoulder to shoulder while the girl tried to gently catch and tug on his talons.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">He only rarely let her win, clearly doing so in order to keep her interested, as otherwise there was no way that she’d have succeeded even a single time.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala, for her part, watched Master Grediv. She honestly didn’t </span><em class=\"calibre4\"><span class=\"calibre3\">think</span></em><span class=\"calibre3\"> that the man would be a threat, but she wasn’t going to take her daughter’s safety for granted.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Some of the surrounding patrons had glanced their way as their group had entered, and a few seemed intrigued by Lea—or to have recognized Tala or Rane—but no one was staring enough to be uncomfortable or rude.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The fights being displayed were all recordings, so Tala assumed that the battle with the electric sheep had concluded in the time it had taken them to get to the city, gain entry, set up Ironhold’s exit, and walk to this establishment.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">On one slate, a man was fighting a bear, bare handed. The creature stood at over fifteen feet, and its fur was a striated mix of black, brown, and white. The magic it used seemed to be a combination of ice and earth, but the Mage boxing against it simply slid and dodged around any dangerous workings, consistently getting knuckles of bear flesh.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><em class=\"calibre4\"><span class=\"calibre3\">-That’s a bad pun, and you should feel bad.-</span></em></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><em class=\"calibre4\"><span class=\"calibre3\">I don’t know what you’re talking about.</span></em><span class=\"calibre3\"> Tala still sent the feeling of a smirk toward her alternate interface.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><em class=\"calibre4\"><span class=\"calibre3\">-Sure, sure.-</span></em></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Another slate showed a group surrounding something that looked like an armored ox. Instead of hooves it had large, strong and stable feet, and two of its four horns were centered, anchored in the long snout.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">It also had wicked teeth that gnashed and tried to tear into those surrounding and harrying it.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Some others showed swarm monsters, one even depicting fliers—which were blessedly rare in general—and over all, the fights were as diverse as Tala would expect.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">It was clear that the closest tables were mostly focused on the slate nearest to them, likely having influenced what was being shown on it. Tala was also able to see that the sound for each particular recording was being given to some of the tables but not all.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane returned, and his arrival seemed to pull Master Grediv back to the present moment. The older man cleared his throat, and he nodded, smiling. “I see. They were quite right. Lea is your daughter through and through, a gated human if ever I’ve seen such a soul.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea practically wiggled with happiness at the words.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The Paragon continued. “She also carries the Gredial… boon.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala and Rane shared a look. He sighed. “We thought that maybe with our official changing of our last name…”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv huffed a laugh. “If it were that easy, it wouldn’t have stuck around for so long. She is a true descendent and thus has the soul impression.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">They gave slow nods of understanding. When Lea asked if it was the berserker issue they’d spoken of briefly, they acknowledged that it was, and promised to speak more on it later.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">After a momentary pause, the elder man cleared his throat and spoke again. “As a gated soul, I assume that she’ll be tested for magic competency?&#34;</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala grinned. “We already have, if unofficially.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“Oh?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea turned to regard her mother as well. “Oh? I didn’t know that.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane smiled at his daughter. “Magic competency is basically a combination of how you think and how well you are able to truly embrace an idea—and all its implications—without actually believing it.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea frowned. “But… That’s just thinking through something thoroughly.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv chuckled. “In a sense you are right, child, but not everyone sees it that way. Many think in different ways, making it more difficult for them to truly learn to master magic. They can still learn if they wish, but their road will be longer and more difficult, and the results likely worse.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea gave a slow nod. “So, I pass, right? I have that?” She looked between her parents, expectantly. “Right?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala grinned. “Yes. You will be fully capable of learning magic if that is your desire.” She hesitated before adding internally, </span><em class=\"calibre4\"><span class=\"calibre3\">Assuming nothing unforeseen prevents it.</span></em></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea lifted her hands in triumph. “Woo!” Then, she froze, looking around in horror at her loud noise. Obviously, no one had heard, but even still, she lowered her arms sheepishly. “I mean. I’m glad.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv chuckled again. “Your Lea seems like a delight.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“I am.” Lea smiled unabashedly.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">He considered then nodded. “I’ve decided, then.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala frowned in confusion, and Rane stiffened, eyes widening.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv smiled. “I’ve been needing another apprentice. Lea will do nicely.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea frowned. “What?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala was already shaking her head, Terry was squawking in annoyance, and Rane was beginning to hold up his hands placatingly when Master Grediv continued. “She is, of course, too young at the present time. I wouldn’t dream of taking her from you, but when she’s older? When she’s ready? And if she wants? I will happily teach her.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">That caused everyone to pause… everyone but Lea. “Teach me?” She looked to her parents. “He wants to teach me magic?” She turned to Master Grediv. “You want to teach me magic?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">He nodded. “I do, when you are ready and if you are interested. Sometimes, it is difficult for parents to teach certain lessons. You can learn them once you are out in the world, or from someone else.” He shrugged. “I’m sure that your parents </span><em class=\"calibre4\"><span class=\"calibre3\">could</span></em><span class=\"calibre3\"> teach you magic if that is your and their wish. You’d also be welcome at the Academy. Though…” He frowned. “I’m honestly uncertain how well you would teleport.” He glanced toward Tala. “That is likely something </span><em class=\"calibre4\"><span class=\"calibre3\">well</span></em><span class=\"calibre3\"> worth investigating.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Tala frowned. “Yes, that… I hadn’t really thought about that.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“Regardless, I am simply stating that: Should you be amenable, </span><em class=\"calibre4\"><span class=\"calibre3\">when</span></em><span class=\"calibre3\"> you are amenable, I am open to teaching you as I taught your father.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">She perked up at that. “You taught dad?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane nodded, smiling once more. “He did. I did not like some of his methods, but looking back, I can see the value in each lesson he imparted.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv grunted. “Not the highest of praise, but I’ll take it.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane chuckled. “It is what it is. I may give you an even higher recommendation after another few decades of intro- and retrospection.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“True enough, my boy, true enough.” Master Grediv grinned in return.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The servers began arriving then, and he lowered the privacy bubble to signal it was alright for them to come in.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane had ordered </span><em class=\"calibre4\"><span class=\"calibre3\">quite</span></em><span class=\"calibre3\"> the spread, including a few different kinds of ribs, causing Lea to </span><em class=\"calibre4\"><span class=\"calibre3\">almost</span></em><span class=\"calibre3\"> squeal in glee. “Thank you, daddy!”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“You’re welcome, sweet.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">After everything was laid out, the servers verified that nothing else was needed and departed.</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">The privacy bubble reactivated and the five dug into the feast. Once the first quick bites were enjoyed, and things started to settle down, Master Grediv caught Rane’s attention. “So, you are going to see your parents as well? I hear that some of your siblings are at the Gredial estate as well.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane swallowed his mouthful and nodded. “Yes. We’ll be staying there for a few days to let them get to know Lea and she, them.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Master Grediv huffed. “I understand the sentiment, and I know I was grateful to spend time with my own grandchildren, but don’t let them spoil the girl, alright?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Lea looked at her father, meeting his gaze full on. “Dad?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">Rane frowned, turning to more fully regard her. “Yes, Lea?”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">“Let them spoil me. That sounds wonderful.”</span></p><p class=\"cnnmmda2owexn2nhmtriotjinzqxzdnlnzu1zdhinjc\"><span class=\"calibre3\">It took a moment or two for the laughter to fade before the conversation continued, wandering through various light topics as the family simply enjoyed their time together.</span></p></div>".to_string(),
        _ =>  "<p>Default chapter content.</p>".to_string(),        
    };

    Ok(chapter)
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