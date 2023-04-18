use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, Path};

struct ReadBuilder {
    last_read_id: usize,
    tracking_ident: Path,
    read_storage: Path,
    tracking_structs: Vec<Read>,
    body: TokenStream,
}

impl ReadBuilder {
    fn dynamic(&mut self, tracking_ident: Ident) -> Read {
        let tracking_ident = &self.tracking_ident;
        let id = self.last_read_id;
        let read_storage = &self.read_storage;
        self.last_read_id += 1;
        let id_bits = 1 << id;
        Read::Dynamic(quote! {#read_storage & #id_bits != 0})
    }

    fn body(&mut self, body: TokenStream) {
        self.body = body;
    }
}

impl ToTokens for ReadBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let body = &self.body;
        let tracking_structs = &self.tracking_structs;
        let tracking_ident = &self.tracking_ident;
        let read_storage = &self.read_storage;
        tokens.extend(quote! {
            {
                #tracking_ident.reset_read();
                #(#tracking_structs)*
                #body
                #read_storage.set(#tracking_ident.get_read());
            }
        });
    }
}

struct Tracking {
    id: usize,
    path: Path,
    tracking_path: Path,
}

impl ToTokens for Tracking {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = self.id;
        let ident = &self.path;
        let tracking_ident = &self.tracking_path;
        tokens.extend(quote! {
            let mut #ident = RwTrack {
                data: &mut #ident,
                tracking: #tracking_ident.track(#id),
            };
        });
    }
}

enum Read {
    Static(bool),
    Dynamic(TokenStream),
}

impl ToTokens for Read {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Read::Static(b) => {
                tokens.extend(quote! {
                    #b
                });
            }
            Read::Dynamic(ts) => {
                tokens.extend(quote! {
                    #ts
                });
            }
        }
    }
}
