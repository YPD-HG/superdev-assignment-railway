use axum::{Json, http::StatusCode, extract::Query};
use serde::{Serialize, Deserialize};
use solana_sdk::{
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::{collections::HashMap, str::FromStr};
use bs58;
use spl_token;

//
// Shared Types
//

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

//
// /keypair
//

#[derive(Serialize)]
pub struct KeypairResponse {
    pub pubkey: String,
    pub secret: String,
}

pub async fn generate_keypair(Query(params): Query<HashMap<String, String>>) 
    -> Result<Json<SuccessResponse<KeypairResponse>>, (StatusCode, Json<ErrorResponse>)> 
{
    if let Some(f) = params.get("fail") {
        if f == "true" {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Simulated failure via query param".to_string(),
                }),
            ));
        }
    }

    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    Ok(Json(SuccessResponse {
        success: true,
        data: KeypairResponse { pubkey, secret },
    }))
}

//
// /token/create
//

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub mintAuthority: String,
    pub mint: String,
    pub decimals: u8,
}

#[derive(Serialize)]
pub struct AccountMetaResponse {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Serialize)]
pub struct CreateTokenResponse {
    pub program_id: String,
    pub accounts: Vec<AccountMetaResponse>,
    pub instruction_data: String,
}

pub async fn create_token(
    Json(req): Json<CreateTokenRequest>,
) -> Result<Json<SuccessResponse<CreateTokenResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(p) => p,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid mint pubkey".into(),
                }),
            ))
        }
    };

    let mint_authority = match Pubkey::from_str(&req.mintAuthority) {
        Ok(p) => p,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid mint authority pubkey".into(),
                }),
            ))
        }
    };

    let token_program_id = spl_token::ID;

    let instruction = spl_token::instruction::initialize_mint(
        &token_program_id,
        &mint_pubkey,
        &mint_authority,
        None,
        req.decimals,
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: format!("Failed to create instruction: {}", e),
            }),
        )
    })?;

    let accounts: Vec<AccountMetaResponse> = instruction
        .accounts
        .into_iter()
        .map(|meta| AccountMetaResponse {
            pubkey: meta.pubkey.to_string(),
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        })
        .collect();

    Ok(Json(SuccessResponse {
        success: true,
        data: CreateTokenResponse {
            program_id: instruction.program_id.to_string(),
            accounts,
            instruction_data: base64::encode(instruction.data),
        },
    }))
}

#[derive(Deserialize)]
pub struct MintTokenRequest {
    pub mint: String,
    pub destination: String,
    pub authority: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct MintTokenResponse {
    pub program_id: String,
    pub accounts: Vec<AccountMetaResponse>,
    pub instruction_data: String,
}

pub async fn mint_token(
    Json(req): Json<MintTokenRequest>,
) -> Result<Json<SuccessResponse<MintTokenResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let mint = Pubkey::from_str(&req.mint).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid mint address".into(),
            }),
        )
    })?;

    let destination = Pubkey::from_str(&req.destination).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid destination address".into(),
            }),
        )
    })?;

    let authority = Pubkey::from_str(&req.authority).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid authority address".into(),
            }),
        )
    })?;

    let instruction = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &destination,
        &authority,
        &[],
        req.amount,
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: format!("Failed to create instruction: {}", e),
            }),
        )
    })?;

    let accounts = instruction
        .accounts
        .into_iter()
        .map(|a| AccountMetaResponse {
            pubkey: a.pubkey.to_string(),
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect();

    let response = MintTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: base64::encode(instruction.data),
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}

#[derive(Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    pub secret: String,
}

#[derive(Serialize)]
pub struct SignMessageResponse {
    pub signature: String,
    pub public_key: String,
    pub message: String,
}

pub async fn sign_message(
    Json(req): Json<SignMessageRequest>,
) -> Result<Json<SuccessResponse<SignMessageResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let secret_bytes = bs58::decode(&req.secret)
        .into_vec()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid base58 secret key".into(),
                }),
            )
        })?;

    let keypair = Keypair::from_bytes(&secret_bytes).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Failed to deserialize secret key".into(),
            }),
        )
    })?;

    let message_bytes = req.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);

    Ok(Json(SuccessResponse {
        success: true,
        data: SignMessageResponse {
            signature: base64::encode(signature),
            public_key: keypair.pubkey().to_string(),
            message: req.message,
        },
    }))
}

#[derive(Deserialize)]
pub struct VerifyMessageRequest {
    pub message: String,
    pub signature: String,
    pub pubkey: String,
}

#[derive(Serialize)]
pub struct VerifyMessageResponse {
    pub valid: bool,
    pub message: String,
    pub pubkey: String,
}

pub async fn verify_message(
    Json(req): Json<VerifyMessageRequest>,
) -> Result<Json<SuccessResponse<VerifyMessageResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let pubkey = Pubkey::from_str(&req.pubkey).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid pubkey".into(),
            }),
        )
    })?;

    let signature_bytes = base64::decode(&req.signature).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid base64 signature".into(),
            }),
        )
    })?;

    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid signature format".into(),
            }),
        )
    })?;

    let dalek_pubkey = ed25519_dalek::PublicKey::from_bytes(pubkey.as_ref()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid public key format".into(),
            }),
        )
    })?;

    let valid = dalek_pubkey
        .verify_strict(req.message.as_bytes(), &signature)
        .is_ok();

    Ok(Json(SuccessResponse {
        success: true,
        data: VerifyMessageResponse {
            valid,
            message: req.message,
            pubkey: req.pubkey,
        },
    }))
}

#[derive(Deserialize)]
pub struct SendSolRequest {
    pub from: String,
    pub to: String,
    pub lamports: u64,
}

#[derive(Serialize)]
pub struct SendSolResponse {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub instruction_data: String,
}

pub async fn send_sol(
    Json(req): Json<SendSolRequest>,
) -> Result<Json<SuccessResponse<SendSolResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let from_pubkey = Pubkey::from_str(&req.from).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid 'from' address".into(),
            }),
        )
    })?;

    let to_pubkey = Pubkey::from_str(&req.to).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid 'to' address".into(),
            }),
        )
    })?;

    let instruction = solana_sdk::system_instruction::transfer(
        &from_pubkey,
        &to_pubkey,
        req.lamports,
    );

    let accounts = instruction
        .accounts
        .iter()
        .map(|meta| meta.pubkey.to_string())
        .collect::<Vec<_>>();

    let response = SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: base64::encode(instruction.data),
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}

#[derive(Deserialize)]
pub struct SendTokenRequest {
    pub destination: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct SendTokenResponse {
    pub program_id: String,
    pub accounts: Vec<AccountMetaSimple>,
    pub instruction_data: String,
}

#[derive(Serialize)]
pub struct AccountMetaSimple {
    pub pubkey: String,
    pub isSigner: bool,
}

pub async fn send_token(
    Json(req): Json<SendTokenRequest>,
) -> Result<Json<SuccessResponse<SendTokenResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Parse all input pubkeys
    let destination = Pubkey::from_str(&req.destination).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid destination address".into(),
            }),
        )
    })?;

    let mint = Pubkey::from_str(&req.mint).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid mint address".into(),
            }),
        )
    })?;

    let owner = Pubkey::from_str(&req.owner).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid owner address".into(),
            }),
        )
    })?;

    // 👇 In transfer_checked, source is owner's associated token account.
    let source = Pubkey::from_str(&req.destination).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid source token address".into(),
            }),
        )
    })?;

    let instruction = spl_token::instruction::transfer_checked(
        &spl_token::ID,
        &source,
        &mint,
        &destination,
        &owner,
        &[],              // multisig signer pubkeys if any
        req.amount,
        6,                // decimals (defaulting to 6)
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: format!("Instruction error: {}", e),
            }),
        )
    })?;

    let accounts = instruction
        .accounts
        .into_iter()
        .map(|meta| AccountMetaSimple {
            pubkey: meta.pubkey.to_string(),
            isSigner: meta.is_signer,
        })
        .collect();

    Ok(Json(SuccessResponse {
        success: true,
        data: SendTokenResponse {
            program_id: instruction.program_id.to_string(),
            accounts,
            instruction_data: base64::encode(instruction.data),
        },
    }))
}
