use crate::Client;
use haya_collection::List;
use haya_protocol::clientbound::{ConfigurationHandler, common, configuration, cookie};
use haya_protocol::serverbound::common::{ConfigurationKeepAlive, KeepAlive};
use haya_protocol::serverbound::configuration::{FinishConfiguration, SelectKnownPacks};
use mser::{Read, Reader, V21};
use tokio::io::AsyncReadExt;

pub async fn handle_configuration(
    client: &mut Client,
    b: &mut Vec<u8>,
    c: &mut usize,
) -> tokio::io::Result<()> {
    let mut h = Configuration {
        finish: false,
        c: client,
    };
    loop {
        let mut flag = true;
        let (read, p) = loop {
            if *c == b.len() {
                b.clear();
                *c = 0;
            }
            if !flag {
                b.reserve(4096);
                h.c.s.read_buf(b).await?;
            }
            flag = false;
            let buf = &b[*c..];
            let ptr = buf.as_ptr();
            let mut r = Reader::new(buf);
            let len = match V21::read(&mut r) {
                Ok(x) => x.0 as usize,
                Err(_) => continue,
            };
            match r.read_slice(len) {
                Ok(x) => unsafe { break (r.offset_from(ptr), x) },
                Err(_) => continue,
            }
        };
        *c += read;
        let p = Reader::new(p);
        h.handle(p).unwrap();
        if !h.c.b.is_empty() {
            h.c.flush().await?;
        }
        if h.finish {
            break;
        }
    }
    Ok(())
}

pub struct Configuration<'a> {
    finish: bool,
    c: &'a mut Client,
}

impl<'a> ConfigurationHandler for Configuration<'a> {
    fn cookie_request(&mut self, _: cookie::ConfigurationCookieRequest<'_>) {}
    fn custom_payload(&mut self, _: common::ConfigurationCustomPayload<'_>) {}
    fn disconnect(&mut self, _: common::ConfigurationDisconnect) {}
    fn finish_configuration(&mut self, _: configuration::FinishConfiguration) {
        self.finish = true;
        self.c.write(&FinishConfiguration {});
    }
    fn keep_alive(&mut self, p: common::ConfigurationKeepAlive) {
        self.c
            .write(&ConfigurationKeepAlive(KeepAlive { id: p.0.id }));
    }
    fn ping(&mut self, _: common::ConfigurationPing) {}
    fn reset_chat(&mut self, _: common::ResetChat) {}
    fn registry_data(&mut self, _: configuration::RegistryData<'_>) {}
    fn resource_pack_pop(&mut self, _: common::ResourcePackPop) {}
    fn resource_pack_push(&mut self, _: common::ResourcePackPush<'_>) {}
    fn store_cookie(&mut self, _: common::ConfigurationStoreCookie<'_>) {}
    fn transfer(&mut self, _: common::ConfigurationTransfer<'_>) {}
    fn update_enabled_features(&mut self, _: configuration::UpdateEnabledFeatures<'_>) {}
    fn update_tags(&mut self, _: common::ConfigurationUpdateTags<'_>) {}
    fn select_known_packs(&mut self, p: configuration::SelectKnownPacks<'_>) {
        self.c.write(&SelectKnownPacks {
            known_packs: List::Borrowed(p.known_packs.as_slice()),
        });
    }
    fn custom_report_details(&mut self, _: common::ConfigurationCustomReportDetails<'_>) {}
    fn server_links(&mut self, _: common::ConfigurationServerLinks<'_>) {}
    fn clear_dialog(&mut self, _: common::ConfigurationClearDialog) {}
    fn show_dialog(&mut self, _: common::ConfigurationShowDialog) {}
    fn code_of_conduct(&mut self, _: configuration::CodeOfConduct<'_>) {}
}
