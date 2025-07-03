use ratatui::style::{Color, Stylize};
use ratatui::text::Span;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse {
  pub page_props: PageProps,
}

impl PageResponse {
  pub fn into_records(self) -> Vec<Record> {
    self.page_props.server_response.data.records
  }
  pub fn as_records(&self) -> &[Record] {
    &self.page_props.server_response.data.records
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProps {
  pub server_response: ServerResponse,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerResponse {
  pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
  pub records: Vec<Record>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
  #[serde(default)]
  pub host_provider: Vec<String>,
  #[serde(default)]
  pub hostname: String,
  #[serde(default)]
  pub mail_provider: Vec<Option<String>>,
  #[serde(default)]
  pub open_page_rank: Option<isize>,
}

impl Record {
  pub fn as_spans(&self) -> [Span; 10] {
    [
      Span::raw("🌐"),
      Span::raw(" "),
      Span::raw(self.hostname.as_str()).fg(Color::Blue),
      Span::raw(" "),
      Span::raw("("),
      Span::raw(self.host_provider.first().map(String::as_str).unwrap_or("❔"))
        .fg(Color::Rgb(0, 255, 174))
        .italic(),
      Span::raw(")"),
      Span::raw(" "),
      Span::raw("📈"),
      Span::raw(self.mail_provider.first().map(|prov| prov.as_deref().unwrap_or("")).unwrap_or("❔")),
    ]
  }

  pub fn as_csv(&self) -> String {
    format!(
      "{host_provider},{},{mail_provider},{rank}",
      self.hostname,
      host_provider = self.host_provider.join("|"),
      mail_provider = self
        .mail_provider
        .clone()
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect::<Vec<_>>()
        .join("|"),
      rank = self.open_page_rank.unwrap_or_default()
    )
  }

  pub fn csv_header() -> &'static str {
    "host_provider,domain,mail_provider,rank"
  }
}

#[test]
fn ttest_resppsone() {
  let value = r###"{"pageProps":{"appContentClasses":"grid p-6 grid-cols-1 gap-4 content-baseline min-h-screen","keyword":"team","locationPathname":"/list/keyword/team","locationSearch":"?page=3","page":3,"searchValue":"team","serverResponse":{"headers":{"server":"nginx","date":"Wed, 02 Jul 2025 20:49:50 GMT","content-type":"application/json; charset=utf-8","content-length":"12257","connection":"close","vary":"Accept-Encoding","access-control-allow-credentials":"true","access-control-allow-origin":"*","access-control-expose-headers":"","asn-risk-level":"none","cache-control":"max-age=0, private, must-revalidate","x-request-id":"GE6KUf-rcySXs48Emlxh","x-frame-options":"SAMEORIGIN","x-content-type-options":"nosniff"},"asnRiskLevel":"none","data":{"meta":{"limit_reached":true,"max_page":100,"page":3,"total_pages":100},"records":[{"host_provider":["Amazon.com, Inc."],"hostname":"ateam-entertainment.com","mail_provider":[],"open_page_rank":191321},{"host_provider":["WEDOS Internet, a.s."],"hostname":"team.urbandroid.org","mail_provider":[],"open_page_rank":191977},{"host_provider":["Automattic, Inc"],"hostname":"preservedbritishsteamlocomotives.com","mail_provider":[],"open_page_rank":192194},{"host_provider":["DigitalOcean, LLC"],"hostname":"gamedevteam.com","mail_provider":["SoftLayer Technologies Inc."],"open_page_rank":192434},{"host_provider":["Cloudflare, Inc."],"hostname":"practiceteam.fr","mail_provider":[],"open_page_rank":193927},{"host_provider":["Amazon.com, Inc."],"hostname":"teamtapas.com","mail_provider":[],"open_page_rank":194310},{"host_provider":["Cloudflare, Inc."],"hostname":"support.team17.com","mail_provider":[],"open_page_rank":195438},{"host_provider":["WebSupport s.r.o."],"hostname":"team2swift.com","mail_provider":[],"open_page_rank":195521},{"host_provider":["Fastly, Inc."],"hostname":"teammobile.io","mail_provider":["Google LLC"],"open_page_rank":195693},{"host_provider":["Google LLC"],"hostname":"allyteam.ru","mail_provider":[],"open_page_rank":195723},{"host_provider":["Amazon.com, Inc."],"hostname":"drive.teamlocus.com","mail_provider":[],"open_page_rank":195816},{"host_provider":["Amazon.com, Inc."],"hostname":"manageteamz.com","mail_provider":["Google LLC"],"open_page_rank":196600},{"host_provider":["home.pl S.A."],"hostname":"blooberteam.com","mail_provider":["Microsoft Corporation"],"open_page_rank":196643},{"host_provider":["Microsoft Corporation"],"hostname":"teamhaven.com","mail_provider":["Google LLC"],"open_page_rank":196977},{"host_provider":["Amazon.com, Inc."],"hostname":"docnowteam.slack.com","mail_provider":[],"open_page_rank":197250},{"host_provider":["Automattic, Inc"],"hostname":"teamqueens.org","mail_provider":[],"open_page_rank":197499},{"host_provider":["Liquid Web, L.L.C"],"hostname":"momsteam.com","mail_provider":["Liquid Web, L.L.C"],"open_page_rank":197742},{"host_provider":["Automattic, Inc"],"hostname":"xfactorteamroping.com","mail_provider":["Google LLC"],"open_page_rank":197819},{"host_provider":["Fastly, Inc."],"hostname":"vteam.com","mail_provider":["Apple Inc.","Microsoft Corporation"],"open_page_rank":199158},{"host_provider":["SECURED SERVERS LLC","Cogent Communications"],"hostname":"steampoweredfamily.com","mail_provider":["Cogent Communications"],"open_page_rank":201298},{"host_provider":["Cloudflare, Inc."],"hostname":"teamsdesign.com","mail_provider":[],"open_page_rank":201299},{"host_provider":["SAKURA Internet Inc."],"hostname":"teammoko.com","mail_provider":["SAKURA Internet Inc."],"open_page_rank":201585},{"host_provider":["dogado GmbH"],"hostname":"teamsports2.de","mail_provider":["dogado GmbH"],"open_page_rank":201618},{"host_provider":["Automattic, Inc"],"hostname":"tryhardteam.com","mail_provider":["Google LLC"],"open_page_rank":202818},{"host_provider":["OVH SAS"],"hostname":"amorphteam.com","mail_provider":["OVH SAS"],"open_page_rank":203011},{"host_provider":["OVH SAS"],"hostname":"pinpinteam.com","mail_provider":["Google LLC"],"open_page_rank":203846},{"host_provider":["Cloudflare, Inc."],"hostname":"teamfit.eu","mail_provider":["Google LLC"],"open_page_rank":203874},{"host_provider":["Wix.com Ltd."],"hostname":"mft-bodyteamwork.com","mail_provider":["Microsoft Corporation"],"open_page_rank":203940},{"host_provider":["Namecheap, Inc."],"hostname":"khadamateama.com","mail_provider":["Namecheap, Inc."],"open_page_rank":204080},{"host_provider":["DigitalOcean, LLC"],"hostname":"teamtvilling.dk","mail_provider":["Google LLC"],"open_page_rank":204204},{"host_provider":["Hetzner Online GmbH"],"hostname":"teamsystems.de","mail_provider":["GoDaddy"],"open_page_rank":204376},{"host_provider":["Amazon.com, Inc."],"hostname":"ar.team","mail_provider":["Google LLC"],"open_page_rank":204438},{"host_provider":["Amazon.com, Inc."],"hostname":"teamterriblegames.com","mail_provider":["Google LLC"],"open_page_rank":204945},{"host_provider":["Register.IT S.p.A."],"hostname":"7team.it","mail_provider":["Microsoft Corporation"],"open_page_rank":205034},{"host_provider":["OVH SAS"],"hostname":"vikings.pinpinteam.com","mail_provider":[],"open_page_rank":205491},{"host_provider":["Cloudflare, Inc."],"hostname":"quazarteam.pro","mail_provider":[],"open_page_rank":205876},{"host_provider":["SCALEWAY S.A.S."],"hostname":"cpiteamgames.com","mail_provider":["Namecheap, Inc."],"open_page_rank":207846},{"host_provider":["Fastly, Inc."],"hostname":"dottie-teams.firebaseapp.com","mail_provider":[],"open_page_rank":208047},{"host_provider":["Google LLC"],"hostname":"inteamchat.com","mail_provider":[],"open_page_rank":208303},{"host_provider":[],"hostname":"teamneolife.mysecureoffice.com","mail_provider":[],"open_page_rank":208648},{"host_provider":[],"hostname":"quazarteam.com","mail_provider":[],"open_page_rank":208883},{"host_provider":["SAKURA Internet Inc."],"hostname":"team-frog.com","mail_provider":["SAKURA Internet Inc."],"open_page_rank":209171},{"host_provider":["Amazon.com, Inc."],"hostname":"teambleet.com","mail_provider":["Google LLC"],"open_page_rank":209172},{"host_provider":["MEMSET Ltd"],"hostname":"teaminusapp.com","mail_provider":["MEMSET Ltd"],"open_page_rank":209173},{"host_provider":["Hostinger International Ltd."],"hostname":"greekteam.gr","mail_provider":["Cloudflare, Inc."],"open_page_rank":209768},{"host_provider":["Cloudflare, Inc."],"hostname":"classteam.io","mail_provider":["Google LLC"],"open_page_rank":209887},{"host_provider":["Fastly, Inc."],"hostname":"gameteamhw.github.io","mail_provider":[],"open_page_rank":209904},{"host_provider":["SAKURA Internet Inc."],"hostname":"teamporali.jp","mail_provider":["SAKURA Internet Inc."],"open_page_rank":210056},{"host_provider":["Wix.com Ltd."],"hostname":"footballteamcenter.com","mail_provider":["Google LLC"],"open_page_rank":211707},{"host_provider":["DigitalOcean, LLC"],"hostname":"infiniteambient.com","mail_provider":["Microsoft Corporation"],"open_page_rank":211893},{"host_provider":[],"hostname":"pp.kosmosteam.com","mail_provider":[],"open_page_rank":211982},{"host_provider":["Squarespace, Inc."],"hostname":"onteambloom.com","mail_provider":["Microsoft Corporation"],"open_page_rank":212236},{"host_provider":["Amazon.com, Inc."],"hostname":"api.teambleet.com","mail_provider":[],"open_page_rank":212557},{"host_provider":["Weebly, Inc."],"hostname":"appteam.jlabs.me","mail_provider":[],"open_page_rank":213426},{"host_provider":["DigitalOcean, LLC"],"hostname":"cloudteam.pro","mail_provider":["YANDEX LLC"],"open_page_rank":213761},{"host_provider":["Cloudflare, Inc."],"hostname":"find-friends-team.ru","mail_provider":["YANDEX LLC"],"open_page_rank":213803},{"host_provider":[],"hostname":"iris-team.ru","mail_provider":[],"open_page_rank":213809},{"host_provider":[],"hostname":"rastishka.ar.team","mail_provider":[],"open_page_rank":213933},{"host_provider":["Amazon.com, Inc."],"hostname":"devcorner.team","mail_provider":[],"open_page_rank":213934},{"host_provider":["The University of Seoul"],"hostname":"uoslife.team","mail_provider":["Kakao Corp"],"open_page_rank":213935},{"host_provider":["IP Backbone of myLoc managed IT AG"],"hostname":"team-mediaportal.com","mail_provider":["GoDaddy"],"open_page_rank":216698},{"host_provider":["Amazon.com, Inc."],"hostname":"usskiteam.com","mail_provider":[],"open_page_rank":218876},{"host_provider":["23M GmbH"],"hostname":"teamdeutschland.de","mail_provider":["Hetzner Online GmbH"],"open_page_rank":219040},{"host_provider":["New Dream Network, LLC"],"hostname":"theteamplays.org","mail_provider":["Google LLC"],"open_page_rank":219327},{"host_provider":["Amazon.com, Inc."],"hostname":"americanqueensteamboatcompany.com","mail_provider":[],"open_page_rank":220047},{"host_provider":["Fastly, Inc."],"hostname":"proteinswebteam.github.io","mail_provider":[],"open_page_rank":220916},{"host_provider":["Liquid Web, L.L.C"],"hostname":"fieldinnovationteam.org","mail_provider":["Rackspace Hosting"],"open_page_rank":220948},{"host_provider":["Hetzner Online GmbH"],"hostname":"steamindex.com","mail_provider":[],"open_page_rank":222561},{"host_provider":["Corporation Service Company"],"hostname":"teamultra.com","mail_provider":[],"open_page_rank":223400},{"host_provider":["Cloudflare, Inc."],"hostname":"demo.ninjateam.org","mail_provider":[],"open_page_rank":224480},{"host_provider":["Im Oberen Werk 1"],"hostname":"ideonteam.com","mail_provider":["Im Oberen Werk 1"],"open_page_rank":224841},{"host_provider":["Google LLC"],"hostname":"teambasedlearning.org","mail_provider":["Google LLC"],"open_page_rank":225068},{"host_provider":[],"hostname":"redteamracing.org","mail_provider":[],"open_page_rank":225828},{"host_provider":["A2 Hosting, Inc."],"hostname":"blog.theteamw.com","mail_provider":[],"open_page_rank":228461},{"host_provider":["Cloudflare, Inc."],"hostname":"steamforged.com","mail_provider":["Google LLC"],"open_page_rank":228630},{"host_provider":["Fastly, Inc."],"hostname":"teamdigitale.governo.it","mail_provider":[],"open_page_rank":229839},{"host_provider":["Amazon.com, Inc."],"hostname":"teamlinkt.com","mail_provider":["Google LLC"],"open_page_rank":231206},{"host_provider":["Cloudflare, Inc."],"hostname":"cleanenergyresourceteams.org","mail_provider":["Google LLC"],"open_page_rank":231905},{"host_provider":["Amazon.com, Inc."],"hostname":"longform.watchdog.team","mail_provider":[],"open_page_rank":231987},{"host_provider":["Cloudflare, Inc."],"hostname":"teambuilding.com","mail_provider":["Google LLC"],"open_page_rank":235076},{"host_provider":[],"hostname":"revistanorteamerica.unam.mx","mail_provider":[],"open_page_rank":235365},{"host_provider":["Incapsula Inc"],"hostname":"fxteam.ru","mail_provider":["Google LLC"],"open_page_rank":238107},{"host_provider":["Google LLC"],"hostname":"nationalteamsoficehockey.com","mail_provider":["Google LLC"],"open_page_rank":238756},{"host_provider":["Hostinger International Ltd."],"hostname":"sportsteamhistory.com","mail_provider":["Cloudflare, Inc."],"open_page_rank":239510},{"host_provider":["Cloudflare, Inc."],"hostname":"steampunkjournal.org","mail_provider":[],"open_page_rank":240077},{"host_provider":["Kevin Holly trading as Silent Ghost e.U."],"hostname":"tracker.archiveteam.org","mail_provider":[],"open_page_rank":240494},{"host_provider":["Microsoft Corporation"],"hostname":"teamtracky.com","mail_provider":["Microsoft Corporation"],"open_page_rank":240575},{"host_provider":["Amazon.com, Inc."],"hostname":"flow.team","mail_provider":["Google LLC","Korea Telecom"],"open_page_rank":241549},{"host_provider":["Cloudflare, Inc."],"hostname":"ibsteam.net","mail_provider":["Google LLC"],"open_page_rank":241839},{"host_provider":["Fastly, Inc."],"hostname":"teamspeedkills.com","mail_provider":[null,"Internap Corporation"],"open_page_rank":244568},{"host_provider":[],"hostname":"storefront.steampowered.com","mail_provider":[],"open_page_rank":246581},{"host_provider":["Cloudflare, Inc."],"hostname":"media-folder.ninjateam.org","mail_provider":[],"open_page_rank":251899},{"host_provider":["Charter Communications"],"hostname":"team.net","mail_provider":[],"open_page_rank":251992},{"host_provider":["Corporation Service Company"],"hostname":"teamineos.com","mail_provider":["Proofpoint, Inc."],"open_page_rank":253187},{"host_provider":["Cloudflare, Inc."],"hostname":"steamboateramuseum.org","mail_provider":["Microsoft Corporation"],"open_page_rank":253834},{"host_provider":["Future Publishing Ltd"],"hostname":"classicrock.teamrock.com","mail_provider":[],"open_page_rank":254071},{"host_provider":["Cloudflare, Inc."],"hostname":"facialteam.eu","mail_provider":["Google LLC"],"open_page_rank":254841},{"host_provider":["Cloudflare, Inc."],"hostname":"ninjateam.gitbook.io","mail_provider":[],"open_page_rank":256269},{"host_provider":["Fastly, Inc."],"hostname":"acateamazon.org","mail_provider":["Google LLC"],"open_page_rank":256782},{"host_provider":["Amazon.com, Inc."],"hostname":"teamtailor.com","mail_provider":["Google LLC"],"open_page_rank":256957}],"total":10000,"total_relation":"gte"},"error":"","status":200,"statusText":"OK","success":true,"csrfToken":"","userIp":""},"type":"keyword","user":{"email":"security@iknroleplay.com","isVerified":true,"name":"Dev","packageCode":"api-0","token":"","csrfSecret":"fVv-UChuOMY8bx2hMnCczSBw","features":{"projects":{"default":false},"sb":{"default":false}},"convertPunycode":false}},"__N_SSP":true}"###;
  let ps: PageResponse = serde_json::from_slice(value.as_bytes()).unwrap();
  println!("{:#?}", ps.as_records());
}
