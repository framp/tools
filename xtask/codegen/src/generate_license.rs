use proc_macro2::{Literal, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};
use std::io;
use xtask::*;
use xtask::{project_root, Mode};
use xtask_codegen::update;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseList {
    license_list_version: String,
    licenses: Vec<Licence>,
    release_date: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Licence {
    reference: String,
    is_deprecated_license_id: bool,
    details_url: String,
    reference_number: u16,
    name: String,
    license_id: String,
    see_also: Vec<String>,
    is_osi_approved: bool,
    is_fsf_libre: Option<bool>,
}

const URL: &str =
    "https://raw.githubusercontent.com/spdx/license-list-data/master/json/licenses.json";
pub(crate) fn generate_license(mode: Mode) -> Result<()> {
    let request = ureq::get(URL);
    let result = request.call()?;
    let license_list = result.into_json::<LicenseList>()?;
    let config_root = project_root().join("crates/rome_project/src/license");

    let tokens = create_data(license_list).expect("To write data into file");

    update(
        &config_root.join("generated.rs"),
        &xtask::reformat(tokens.to_string())?,
        &mode,
    )?;

    Ok(())
}

fn create_data(license_list: LicenseList) -> io::Result<TokenStream> {
    let mut list = vec![];

    for item in license_list.licenses {
        let reference = Literal::string(&item.reference);
        let details_url = Literal::string(&item.details_url);
        let name = Literal::string(&item.name);
        let license_id = Literal::string(&item.license_id);
        let is_deprecated_license_id = item.is_deprecated_license_id;
        let is_osi_approved = item.is_osi_approved;
        let is_fsf_libre = item.is_fsf_libre;
        let see_also: Vec<_> = item
            .see_also
            .iter()
            .map(|see_also| {
                let see_also = Literal::string(&see_also);
                quote!(#see_also)
            })
            .collect();
        list.push(quote! {

             &Licence {
                reference: #reference,
                is_deprecated_license_id: #is_deprecated_license_id,
                details_url: #details_url,
                reference_number: 0,
                name: #name,
                license_id: #license_id,
                see_also: &[
                    #( #see_also ),*
                ],
                is_osi_approved: #is_osi_approved,
                is_fsf_libre: #is_fsf_libre,
            }
        });
    }

    let tokens = quote! {

        #[derive(Debug)]
        pub struct LicenseList {
            license_list_version: &'static str,
            license_list: &'static [&'static Licence],
            release_date: &'static str,
        }

        #[derive(Debug)]
        pub struct Licence {
            reference: &'static str,
            is_deprecated_license_id: bool,
            details_url: &'static str,
            reference_number: u16,
            name: &'static str,
            license_id: &'static str,
            see_also: &'static [&'static str],
            is_osi_approved: bool,
            is_fsf_libre: bool,
        }

        let license = quote! {
            #( #list ),*
        };
        const LICENSE_LIST: LicenseList = LicenseList {
            license_list_version: ,
            license_list:  ,
            release_date: &'static str,
        };
    };

    Ok(tokens)
}
