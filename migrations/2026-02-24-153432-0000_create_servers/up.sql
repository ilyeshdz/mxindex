CREATE TABLE servers (
    id BIGSERIAL PRIMARY KEY,
    domain TEXT NOT NULL UNIQUE,
    name TEXT,
    description TEXT,
    logo_url TEXT,
    theme TEXT,
    registration_open BOOLEAN,
    public_rooms_count INTEGER,
    version TEXT,
    federation_version TEXT,
    delegated_server TEXT,
    room_versions TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
