use std::collections::BTreeSet;
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::{Arc, LazyLock};

use cidr::{Cidr, Ipv4Cidr};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIs, FromRepr, VariantArray, VariantNames};

use crate::app::handler::get_ip;

pub static AWS_IP: LazyLock<AwsRanges> = LazyLock::new(|| serde_json::from_reader(fs::File::open("ip-ranges.json").unwrap()).unwrap());

#[derive(Deserialize)]
pub struct AwsRanges {
  pub prefixes: Arc<[Prefix]>,
}

impl AwsRanges {
  pub fn get_prefix(&self, ip: &Ipv4Addr, service: impl Into<Option<IpService>>) -> Option<&Prefix> {
    let service = service.into();
    let s = service.as_ref();
    let mut prefixes = self.prefixes.iter().filter(|p| s.is_none_or(|is| is.eq(&p.service)));

    loop {
      let prefix = prefixes.next()?;
      if prefix.ip_prefix.contains(ip) {
        break Some(prefix);
      }
    }
  }
}

#[derive(Deserialize, Debug)]
pub struct Prefix {
  ip_prefix: Ipv4Cidr,
  service: IpService,
  region: Region,
}

impl Prefix {
  pub fn cidr(&self) -> Ipv4Cidr {
    self.ip_prefix
  }
  pub fn service(&self) -> IpService {
    self.service
  }
  pub fn region(&self) -> Region {
    self.region
  }
}

#[derive(Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, EnumIs)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IpService {
  Amazon,
  ChimeVoiceconnector,
  Route53Healthchecks,
  S3,
  IvsRealtime,
  WorkspacesGateways,
  Ec2,
  Route53,
  Cloudfront,
  Globalaccelerator,
  AmazonConnect,
  Route53HealthchecksPublishing,
  ChimeMeetings,
  CloudfrontOriginFacing,
  Cloud9,
  Codebuild,
  ApiGateway,
  Route53Resolver,
  Ebs,
  Ec2InstanceConnect,
  KinesisVideoStreams,
  AmazonAppflow,
  AuroraDsql,
  MediaPackageV2,
  Dynamodb,
}

macro_rules! enum_rules {
    {
      pub enum $name:ident {
        $($variant:ident $(: $lit:literal)? $(,)?)*
      }
    } => {
        #[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, ::strum::AsRefStr, ::strum::EnumIs, ::strum::Display, ::serde::Serialize, ::serde::Deserialize, ::strum::VariantNames, ::strum::VariantArray)]
        pub enum $name {
          $(
          $(
          #[serde(rename = $lit)]
          #[strum(to_string = $lit)]
          )?
          $variant
          ),*
        }
    };
}

enum_rules! {
  pub enum Region {
    EuWest1: "eu-west-1",
    ApNortheast2: "ap-northeast-2",
    ApEast2: "ap-east-2",
    EuCentral1: "eu-central-1",
    UsEast1: "us-east-1",
    EuWest3: "eu-west-3",
    IlCentral1: "il-central-1",
    EuWest2: "eu-west-2",
    UsWest1: "us-west-1",
    UsWest2: "us-west-2",
    ApSoutheast2: "ap-southeast-2",
    Global: "GLOBAL",
    UsEast2: "us-east-2",
    MeWest1: "me-west-1",
    MxCentral1: "mx-central-1",
    ApSoutheast1: "ap-southeast-1",
    ApSouth1: "ap-south-1",
    ApEast1: "ap-east-1",
    ApSoutheast7: "ap-southeast-7",
    ApNortheast1: "ap-northeast-1",
    MeCentral1: "me-central-1",
    ApSoutheast5: "ap-southeast-5",
    CaCentral1: "ca-central-1",
    CnNorth1: "cn-north-1",
    SaEast1: "sa-east-1",
    EuSouth1: "eu-south-1",
    ApSoutheast3: "ap-southeast-3",
    ApNortheast3: "ap-northeast-3",
    UsGovEast1: "us-gov-east-1",
    UsGovWest1: "us-gov-west-1",
    CnNorthwest1: "cn-northwest-1",
    AfSouth1: "af-south-1",
    CaWest1: "ca-west-1",
    EuNorth1: "eu-north-1",
    MeSouth1: "me-south-1",
    EuSouth2: "eu-south-2",
    ApSoutheast6: "ap-southeast-6",
    EuCentral2: "eu-central-2",
    ApSoutheast4: "ap-southeast-4",
    ApSouth2: "ap-south-2",
    EuscDeEast1: "eusc-de-east-1",
  }
}

impl Region {
  pub fn from_str_contains(value: &str) -> Option<Self> {
    for this in <Self as strum::VariantArray>::VARIANTS {
      if value.to_lowercase().contains(this.as_ref()) {
        return Some(*this);
      }
    }

    None
  }
  pub async fn from_ip(name: &str) -> Option<Self> {
    let domain = addr::parse_domain_name(name).ok()?;
    let ip = get_ip(domain).await?;
    AWS_IP.get_prefix(&ip, None).map(|prefix| prefix.region)
  }
}
impl From<&str> for Region {
  fn from(value: &str) -> Self {
    let value = if value.is_empty() {
      return Self::UsEast1;
    } else {
      value.to_lowercase()
    };

    let variant_str = <Region as strum::VariantNames>::VARIANTS;
    let variant_name = <Region as strum::VariantArray>::VARIANTS;
    let variants = variant_str.iter().zip(variant_name);
    for (variant, region) in variants {
      if **variant == *value {
        return *region;
      }
    }

    Self::UsEast1
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use tokio::time::Instant;

  use super::*;

  #[test]
  fn test_deserialize() {
    let ip: Ipv4Addr = "52.219.178.40".parse().unwrap();
    let start = Instant::now();
    let ranges: AwsRanges = serde_json::from_reader(fs::File::open("ip-ranges.json").unwrap()).unwrap();
    let end = start.elapsed();
    println!("Time take to construct: {end:?}");
    assert_eq!(ranges.prefixes.len(), 8894);

    let start = Instant::now();
    let prefix = ranges.get_prefix(&ip, None);
    let end = start.elapsed();
    println!("Time take to search {ip}: {end:?}");
    assert!(prefix.is_some());

    let prefix = prefix.unwrap();
    assert!(prefix.region.is_us_east_2());
  }
}
