-- Add up migration script here

INSERT INTO plans(
    code , name, price, duration, save_percentage
) VALUES
    ('1_MONTH', 'One Month', 2.0, 30, 0),
    ('3_MONTH', '3 Months', 5.70, 90, 5),
    ('6_MONTH', '6 Months', 9.60, 180, 10),
    ('12_MONTH', '12 Months', 19.20, 365, 20)
;
