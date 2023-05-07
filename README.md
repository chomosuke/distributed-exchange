# distributed-exchange

## Application

- A distributed central database which implements the 'Saga' protocol to facilitate distributed transactions.
  - Users which will interact with the servers via HTTPS.
  - Each node hold multiple accounts.
  - Each process manage one accounts.
- There are < 10^4 stocks.
- There are < 19^9 accounts.
- Each account have roughly 0.1 unmatched buy or sell order at any given time.

## Running instructions

- Launch a single coordinator:
  `cargo run -p coordinator -- -p <port>`
- Launch at least 2 market database servers:
  `cargo run -p node -- -c <coordinator address & port> -p <port>`
- Launch at least 2 clients:
  `cargo run -p client -- -c <coordinator address & port>`

### Running example

- Terminal 1:
  `cargo run -p coordinator -- -p 8000`
- Terminal 2:
  `cargo run -p node -- -c 127.0.0.1:8000 -p 8001`
- Terminal 3:
  `cargo run -p node -- -c 127.0.0.1:8000 -p 8002`
- Terminals 4 and 5:
  `cargo run -p client -- -c 127.0.0.1:8000`

## Approaches

1. Each node have local database (CHOSEN)
   - Each Node have its own copy of all buy and sell order on the market.
   - Every buy and sell order updates that database.
   - Every time the database is updated, the Node tries to create a match where at least one of the buyer or seller is an account that they own.
   - For every buy or sell order, the Node that owns the account where the order came from is the source of truth.
   - Upon matching, the Node takes away the money or stock and quantity in the order and account that they own (do they want to send out that update?) and send out a trade offer to the Node that owns the other account.
   - Upon receiving a trade offer, the Node deducts that money or stock and quantity in the order and account they own and send broadcast a trade confirm. (At this point the trade is confirmed)
     - If upon receiving a trade offer, the Node does not have enough money or stock in the order then they reply with a declined message and the other Node rollback (Roll back is also an update to the database so will attempt to be matched).
   - Both Nodes add stock OR money to the account that they own.
2. MD sends request, awaits replies from all servers for potential match. If successful, execute. Else, hold on to request.
   - Not good because we lose the ability to see a stats of the whole market.

## Example runs of the protocol

Assume M1 and M2 are nodes.

e.g. M1 receives a SELL order from a user. It tries to match locally and fails, so it sends a SELL to all M. => M messages
M1 receives a BUY order from a user. It matches locally with the SELL. It sends a TRADE_CONFIRMED to all M. => M messages

e.g. M1 receives a SELL order from a user. It tries to match locally and fails, so it sends a SELL to all M. => M messages
M2 receives a BUY order from a user. It matches locally with the SELL. It sends a TRADE_OFFER to M1. => 1 message
M1 sends a TRADE_CONFIRMED to all M. => M messages.

## Fault tolerance

When a node crashes, it'll be restarted.\
We store everything important: money, stock, order, pending transaction from their own account.\
When a node restart, or is first started, it'll query every other node to build the local database, and also ask the other node to add it to the update list of database update.\
TODO: update the protocol for crash failure

If a Node already sent out a trade offer, it can't commit or abort without a trade reply.\
If a Node receives a trade offer, it can commit or abort immediately before sending the trade reply.
When a node crash, all the transactions that involve that node can't be committed or aborted. But all the account that node owns can't do anything as well, so it's not that much worse.

## Coordinator

One single dedicated server will listen on an IP address.
All Nodes will contact that server for a list of IP address of the other servers and register its own address.
User login with the coordinator and get IP address from the server.
User establish TCP connection with the Node.

## Messaging protocol

We have 3 executable: Coordinator, Node, Client. Client is optional.

types:

- UserID is a json object of the following structure:
  ```json
  {
    "id": 5,
    "node_id": 3
  }
  ```
- TradeID is an integer.
- Ticker is a string.
- All price are in unit of cents

### Node2Node

There are 3 kinds of message:

- Order: Buy/Sell, ticker, userid, quantity, price.
- TradeOffer: TradeId, ticker, userid_buyer, userid_seller, quantity, price
- TradeRep: Confirmed/Declined, TradeId

Communication channel: TCP stream.

Message format one line per json message:

- Establish connection

- Send node_id (json number)

- If recovered: // UNIMPLEMENTED
  - Both side (recovered side first) send list of all user buy orders and sell orders
  - Both side poll answer for pending request

```json
{
  "type": "order",
  "value": {
    "deduct": false,
    "order_type": "buy|sell",
    "ticker": "Ticker",
    "user_id": "UserID",
    "quantity": 100,
    "price": 1050
  }
}
```

```json
{
  "type": "offer",
  "value": {
    "id": "TradeID",
    "ticker": "Ticker",
    "buyer_id": "UserID",
    "seller_id": "UserID",
    "quantity": 100,
    "price": 1050,
    "buy_price": 1050,
    "sell_price": 1000
  }
}
```

```json
{
  "type": "reply",
  "value": {
    "accepted": true,
    "id": "TradeID"
  }
}
```

### Node2Coordinator

- Register (new or recovered) node
  - Establish connection
    Node -> Coord
  ```json
  {
    "addr": "<node addr>",
    "state": {
      // null if new node
      "id": 3,
      "account_num": 100
    }
  }
  ```
  Coord -> Node
  ```json
  {
    "id": 3 // null if recovered node
    "others": [ // other nodes
      {
        "id": 1,
        "addr": "<node addr>"
      }
    ]
  }
  ```
  Node -> Coord
  ```json
  "ok" // recieved and prepared to connect with all other nodes
  ```
  - req res messages
    - New / recovered node joined:
      req:
    ```json
    {
      "type": "joined",
      "id": 3,
      "addr": "<node addr>"
    }
    ```
    res: No Reply
    - New account request
      req:
    ```json
    { "type": "C account" }
    ```
    res:
    ```json
    "UserID"
    ```

### Client2Coordinator

- Find Node for account.
  - Establish connection
    req:
  ```json
  "UserID"
  ```
  res:
  ```
  "<node addr>" // to be parsed by SocketAddr::parse()
  ```
  - Close connection
- Create accounts.
  - Establish connection
    req:
  ```json
  "C account"
  ```
  res:
  ```json
  "UserID"
  ```
  - Close connection

### Client2Node

- Establish connection
  - Client send UserID
- RU for account balance.

  req:
  ```json
  { "type": "R balance" }
  ```
  res:
  ```json
  100
  ```
  req:
  ```json
  {
    "type": "U balance",
    "value": 100
  }
  ```
  res:
  ```json
  "ok|notEmpty"
  ```
- CR for stocks in account.

  req:
  ```json
  { "type": "R stock" }
  ```
  res:
  ```json
  {
    "tickerID": 100,
    "tickerID2": 200
  }
  ```
  req:
  ```json
  {
    "type": "C stock", // IPO: Should be privileged to admin user of some governing body
    "value": {
      "ticker_id": "tickerID",
      "quantity": 1000
    }
  }
  ```
  res:
  ```
  "ok"
  ```
- R for market status.
  req:
  ```json
  { "type": "R market" }
  ```
  res:
  ```json
  {
    "tickerID": {
      "sell": [
        {
          "quantity": 100,
          "price": 1050
        }
      ],
      "buy": []
    },
    "tickerID2": {
      "sell": [],
      "buy": []
    }
  }
  ```
- CRD for orders.
  req:
  ```json
  {
    "type": "C order",
    "value": {
      "order_type": "buy|sell",
      "ticker": "tickerID",
      "price": 1050,
      "quantity": 100
    }
  }
  ```
  res:
  ```json
  "ok|notEnough"
  ```
  req:
  ```json
  { "type": "R order" }
  ```
  res:
  ```json
  {
    "tickerID": {
      "sell": [
        {
          "quantity": 100,
          "price": 1055
        }
      ],
      "buy": []
    },
    "tickerID2": {
      "sell": [],
      "buy": []
    }
  }
  ```
  req:
  ```json
  {
    "type": "D order",
    "value": {
      "order_type": "buy|sell",
      "ticker": "tickerID",
      "price": 1050,
      "quantity": 100
    }
  }
  ```
  res:
  ```json
  90 // quantity deleted (the rest already traded or didn't exist in the first place)
  ```
- Delete accounts.
  req:
  ```json
  { "type": "D account" }
  ```
  res:
  ```json
  "ok|notEmpty"
  ```
- Terminating Connection:
  ```json
  "bye"
  ```

### Coordinator failure
TODO

## Architecture
- Matcher: one on each node, responsible for matching order where one of the seller or buyer belong to this node.
- Accounts: source of truth, responsible for recording orders, balance, portfolio and stocks.
- There can not be more order than balance / stock in portfolio.

### Matcher syncing
- local order created.
  - matched with remote -> no need to broadcast.
  - matched with local -> broadcast to everyone. TICK
- offer accepted.
  - need to broadcast to everyone except the offer sender. TICK
- local order matched with remote.
  - need to broadcast to everyone. TICK
- Order deducted.
  - need to broadcast. TICK
