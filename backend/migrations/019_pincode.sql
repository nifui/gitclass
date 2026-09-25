CREATE TABLE pincode (
   code_hash TEXT NOT NULL,
   created_at TIMESTAMPTZ NOT NULL DEFAULT now(),  
   expires_at TIMESTAMPTZ NOT NULL
);
