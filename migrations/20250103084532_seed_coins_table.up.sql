-- Add up migration script here

INSERT INTO coins (
    id,
    symbol,
    name,
    logo,
    network,
    price_pair_symbol,
    decimals,
    contract_address,
    description) VALUES 
    (1, 'BTC', 'Bitcoin', '', 'BTC', 'BTC-USDT', 8, NULL, ''),
    (2, 'ETH', 'Ethereum', '', 'ETH', 'ETH-USDT', 18, NULL, ''),
    (3, 'SOL', 'Solana', '', 'SOL', 'SOL-USDT', 9, NULL, ''),
    (4, 'TRX', 'Tron', '', 'TRX', 'TRX-USDT', 6, NULL, ''),
    (5, 'USDT', 'Tether', '', 'ETH', NULL, 6, '0xdAC17F958D2ee523a2206206994597C13D831ec7', ''),
    (6, 'USDT', 'Tether', '', 'TRX', NULL, 6, 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t', '');
