CREATE INDEX idx_servers_domain ON servers(domain);
CREATE INDEX idx_servers_registration_open ON servers(registration_open);
CREATE INDEX idx_servers_created_at ON servers(created_at);
CREATE INDEX idx_servers_public_rooms_count ON servers(public_rooms_count);
