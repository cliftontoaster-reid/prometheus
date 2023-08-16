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

      let message = format!("We hope this message finds you well. We are writing to bring to your attention a matter of immediate importance. Please be advised that we have identified certain links that require your urgent attention. These links are indicated as follows: {}. 

It has come to our attention that the platforms {} have flagged these links due to potential hazards associated with them, primarily linked to concerns related to {}. 
      
In an effort to prioritize user safety and security, we have taken the initiative to modify the links by replacing all occurrences of \"/\" with \">\" in order to render them non-clickable. This additional layer of caution is designed to deter accidental clicks and to enhance the overall safety of recipients.
      
Moreover, we would like to emphasize that our automated system is programmed to never send out links unless explicitly prompted by a command. If you receive any unsolicited link from this system, kindly disregard it as it may have been compromised or manipulated for malicious intent.
      
We sincerely appreciate your swift cooperation in addressing this matter. Your immediate action is invaluable in helping us ensure the continued security and well-being of our recipients.
      
Thank you for your understanding and vigilance in this regard.
      
Best Regards,
      
Clifton Toaster Reid
Prometheus Team", 
        concatenate_with_and(links_show),
        concatenate_with_and(platforms),
        concatenate_with_and(threats));

      msg.reply(ctx, message).await.unwrap();
    }
    malicious
  } else {
    false
  }
}
