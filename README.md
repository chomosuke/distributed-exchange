# distributed-exchange

A distributed stock exchange which implements the '2 Phase Commit' protocol to facilitate distributed transactions, built in Rust.



Application:
- A distributed central database, serving as server for users which will interact with the servers via HTTPS.
    - Each node hold multiple accounts.
    - Each process manage one accounts.
- There are < 10^4 stocks.
- There are < 19^9 accounts.
- Each account have roughly 0.1 unmatched buy or sell order at any given time.

Approaches:
1. Each node have local db
    - Each Node have its own copy of all buy and sell order on the market.
    - Every buy and sell order update that database.
    - Every time the database is updated, the Node tries to create a match where at least one of the buyer or seller is an account that they own.

    - For every buy or sell order, the Node that owns the account where the order came from is the source of truth.

    - Upon matching, the Node takes away the money or stock in the order that they own (do the want to send out that update?) and send out a trade offer to the Node that owns the other account.
    - Upon receiving a trade offer, the Node deducts that money or stock in the order they own and send broadcast a trade confirm. (At this point the trade is confirmed)
        - If upon receiving a trade offer, the Node does not have enough money or stock in the order then they reply with a declined message and the other Node rollback (Roll back is also an update to the database so will attempt to be matched).
    - Both Node add stock / money to the account that they own.






There are X kinds of message:
- Buy order: ticker, userid, quantity, price.
- Sell order: ticker, userid, quantity, price.
- Trade: TradeId, ticker, userid_buyer, userid_seller, quantity, price
- AckTrade: TradeId
- DeclineTrade: TradeId

2 approaches:
- MD tries to match locally. Sends match out to all servers. If successful, execute. Else, broadcast.
- MD sends request, awaits replies from all servers for potential match. If successful, execute. Else, hold on to request.
