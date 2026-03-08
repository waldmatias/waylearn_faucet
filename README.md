# Waylearn Faucet

## What
A short and to the point faucet for people participating in Waylearn's Solana bootcamps.

## Why?
After seeing a ton of messages in the whatsapp groups, seemed like a logical thing to do...
plus, it would also require learning about Anchor, accounts, PDAs, system program and transfers
(basic building blocks for Solana development).

While you can get SOL from the DevNet faucet, it would be great to have an available vault with SOL
for each bootcamp. This vault could be whitelisted, meaning, only people registered could actually 
get SOL from Waylearn's faucet. 

Also, the DevNet faucet is time-bound, while this one is actually amount bound (max top up amount set
at Initialization) and amount sent to the recipient being dynamic. 
The actual sent amount from the vaule to the recipient is `max_topup_amount - current_recipient_balance`. 

## What's next? 
Actually, now that the faucet is working, we just have to link it to _something_ in the whatsapp group
for self-servicing sol drops during the bootcamps :)

Also, whitelisting mechanism, tied to the previous point, would be great to have. 


