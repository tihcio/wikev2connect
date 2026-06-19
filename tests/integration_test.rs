// Integration tests for WIKEv2Connect

#[cfg(test)]
mod tests {
    use WIKEv2Connect::vpn::models::{VpnConfig, VpnProposal};
    use WIKEv2Connect::config;

    #[test]
    fn test_vpn_proposal_creation() {
        let proposal = VpnProposal {
            ike: "aes256-sha256-modp2048".to_string(),
            esp: "aes256-sha1".to_string(),
        };

        assert_eq!(proposal.ike, "aes256-sha256-modp2048");
        assert_eq!(proposal.esp, "aes256-sha1");
    }

    #[test]
    fn test_vpn_config_to_proposal() {
        let config = VpnConfig {
            name: "TestVPN".to_string(),
            server_address: "192.168.1.1".to_string(),
            dh_group: "Group14".to_string(),
            encryption_method: "AES256".to_string(),
            integrity_check: "SHA256".to_string(),
            cipher_transform: "AES256".to_string(),
            auth_transform: "SHA196".to_string(),
        };

        let proposal = config.to_proposal();

        assert_eq!(proposal.ike, "aes256-sha256-modp2048");
        assert_eq!(proposal.esp, "aes256-sha1");
    }

    #[test]
    fn test_vpn_config_with_different_groups() {
        let configs = vec![
            ("Group14", "modp2048"),
            ("Group19", "ecp256"),
            ("Group20", "ecp384"),
        ];

        for (group_in, group_out) in configs {
            let config = VpnConfig {
                name: "Test".to_string(),
                server_address: "1.1.1.1".to_string(),
                dh_group: group_in.to_string(),
                encryption_method: "AES256".to_string(),
                integrity_check: "SHA256".to_string(),
                cipher_transform: "AES256".to_string(),
                auth_transform: "SHA196".to_string(),
            };

            let proposal = config.to_proposal();
            assert!(proposal.ike.contains(group_out));
        }
    }

    #[tokio::test]
    async fn test_parse_powershell_config_basic() {
        let ps_script = r#"
        param(
            [string]$ServerAddress = 'vpn.example.com',
            [string]$Name = 'MyVPN'
        )
        $DHGroup = 'Group14'
        $EncryptionMethod = 'AES256'
        $IntegrityCheckMethod = 'SHA256'
        $CipherTransformConstants = 'AES256'
        $AuthenticationTransformConstants = 'SHA196'
        "#;

        match config::parse_powershell_config(ps_script).await {
            Ok(cfg) => {
                assert_eq!(cfg.name, "MyVPN");
                assert_eq!(cfg.server_address, "vpn.example.com");
                assert_eq!(cfg.dh_group, "Group14");
                assert_eq!(cfg.encryption_method, "AES256");
                assert_eq!(cfg.integrity_check, "SHA256");
            }
            Err(e) => {
                panic!("Failed to parse config: {}", e);
            }
        }
    }

    #[test]
    fn test_connection_status_variants() {
        use WIKEv2Connect::vpn::models::ConnectionStatus;

        let connected = ConnectionStatus::Connected;
        let disconnected = ConnectionStatus::Disconnected;
        let error = ConnectionStatus::Error("Test error".to_string());

        assert_eq!(connected, ConnectionStatus::Connected);
        assert_eq!(disconnected, ConnectionStatus::Disconnected);
        assert_ne!(connected, disconnected);
        match error {
            ConnectionStatus::Error(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Error variant"),
        }
    }
}
