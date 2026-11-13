//! Product identity shared by the desktop application and its engine.
#![forbid(unsafe_code)]

#[cfg(all(feature = "brand-hasisolid", feature = "brand-mssolid"))]
compile_error!("Select only one product brand");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Product {
    pub name: &'static str,
    pub data_folder: &'static str,
    pub xdg_folder: &'static str,
    pub credential_service: &'static str,
    pub app_id: &'static str,
    pub version: &'static str,
    pub branded: bool,
}

#[cfg(feature = "brand-hasisolid")]
pub const PRODUCT: Product = Product {
    name: "HasiSolid",
    data_folder: "HasiSolid",
    xdg_folder: "hasisolid",
    credential_service: "HasiSolid agent chat",
    app_id: "de.hasi-elektronic.hasisolid",
    version: "0.2.4",
    branded: true,
};

#[cfg(all(feature = "brand-mssolid", not(feature = "brand-hasisolid")))]
pub const PRODUCT: Product = Product {
    name: "MSSolid",
    data_folder: "MSSolid",
    xdg_folder: "mssolid",
    credential_service: "MSSolid agent chat",
    app_id: "de.manfred-sickinger.mssolid",
    version: "0.2.4",
    branded: true,
};

#[cfg(not(any(feature = "brand-hasisolid", feature = "brand-mssolid")))]
pub const PRODUCT: Product = Product {
    name: "SolidCraft",
    data_folder: "SolidCraft",
    xdg_folder: "solidcraft",
    credential_service: "SolidCraft agent chat",
    app_id: "io.tangencylabs.solidcraft",
    version: env!("CARGO_PKG_VERSION"),
    branded: false,
};

#[cfg(test)]
mod tests {
    use super::PRODUCT;

    #[test]
    fn identity_matches_the_selected_build() {
        #[cfg(feature = "brand-hasisolid")]
        assert_eq!((PRODUCT.name, PRODUCT.data_folder, PRODUCT.xdg_folder, PRODUCT.branded), ("HasiSolid", "HasiSolid", "hasisolid", true));
        #[cfg(feature = "brand-mssolid")]
        assert_eq!((PRODUCT.name, PRODUCT.data_folder, PRODUCT.xdg_folder, PRODUCT.branded), ("MSSolid", "MSSolid", "mssolid", true));
        #[cfg(not(any(feature = "brand-hasisolid", feature = "brand-mssolid")))]
        assert_eq!((PRODUCT.name, PRODUCT.data_folder, PRODUCT.xdg_folder, PRODUCT.branded), ("SolidCraft", "SolidCraft", "solidcraft", false));
        #[cfg(feature = "brand-hasisolid")]
        assert_eq!(PRODUCT.credential_service, "HasiSolid agent chat");
        #[cfg(feature = "brand-hasisolid")]
        assert_eq!(PRODUCT.app_id, "de.hasi-elektronic.hasisolid");
        #[cfg(feature = "brand-hasisolid")]
        assert_eq!(PRODUCT.version, "0.2.4");
        #[cfg(feature = "brand-mssolid")]
        assert_eq!(PRODUCT.credential_service, "MSSolid agent chat");
        #[cfg(feature = "brand-mssolid")]
        assert_eq!(PRODUCT.app_id, "de.manfred-sickinger.mssolid");
        #[cfg(feature = "brand-mssolid")]
        assert_eq!(PRODUCT.version, "0.2.4");
        #[cfg(not(any(feature = "brand-hasisolid", feature = "brand-mssolid")))]
        assert_eq!(PRODUCT.credential_service, "SolidCraft agent chat");
        #[cfg(not(any(feature = "brand-hasisolid", feature = "brand-mssolid")))]
        assert_eq!(PRODUCT.app_id, "io.tangencylabs.solidcraft");
        #[cfg(not(any(feature = "brand-hasisolid", feature = "brand-mssolid")))]
        assert_eq!(PRODUCT.version, env!("CARGO_PKG_VERSION"));
    }
}
