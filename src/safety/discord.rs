use crate::{safety::check_url, utils::*};
use linkify::LinkFinder;
use serenity::{model::channel::Message, prelude::Context};

pub async fn check_malicious(google_token: &String, ctx: &Context, msg: &Message) -> bool {
  let links_r: Vec<_> = LinkFinder::new().links(&msg.content).collect();

  if links_r.len() > 0 {
    let mut links: Vec<String> = Vec::new();
    let mut links_show: Vec<String> = Vec::new();
    for i in links_r {
      links.push(i.as_str().to_owned());
    }
    let thread_res = check_url(links.clone(), google_token.clone()).await;
    let malicious = thread_res.is_malicious();
    if malicious {
      let mut platforms: Vec<String> = Vec::new();
      let mut threats: Vec<String> = Vec::new();

      for m in &thread_res.matches {
        add_if_not_present(
          &mut &mut platforms,
          &lowercase_and_replace(&m.platform_type.to_owned()),
        );
        add_if_not_present(
          &mut &mut threats,
          &lowercase_and_replace(&m.threat_type.to_owned()),
        );
        links_show.push(m.threat.url.as_str().replace("/", ">").to_owned());
      }

      let links_formated = concatenate_with_and(links_show);
      let platforms_formated = platforms.join("\n- ");
      let threats_formated = concatenate_with_and(threats);

      let message = format!("Greetings and good fortune be upon thee,

May this correspondence reach thee in the best of health and spirits. We humbly scribe to thee, urgently laying forth a matter of gravest concern. It is with utmost respect that we present these identified links for thy immediate consideration:
      
- {links_formated}
      
Our vigilance hath discovered these links, casting shadows of potential peril. The platforms of {platforms_formated}, in their wisdom, have raised warnings of {threats_formated} that may lurk therein.
      
Furthermore, it is of great import to declare that our loyal automaton, by its design, doth not dispatch links without thy explicit directive. Should an unsolicited link assail thy senses, we beseech thee to dismiss it, for it might be tainted with nefarious designs.
      
In gratitude, we commend thy swift action in attending to this pressing matter. Thy response shall stand as a bulwark safeguarding the security and tranquility of our esteemed recipients.
      
With the utmost esteem and consideration,
      
Clifton Toaster Reid
Bearer of the Prometheus Banner");

      msg.reply(&ctx.http, message).await.unwrap();
    }
    malicious
  } else {
    false
  }
}
