pub static EMAIL_STYLESHEET : &str = r#"
<style>
*{
    margin:0;
    padding:0;
    box-sizing:border-box;
}

body{
    font-family:Segoe UI,Tahoma,Geneva,Verdana,sans-serif;
    background:#171a21;
    color:#ecf0f1;
    padding:40px 20px;
}

.container{
    max-width:1100px;
    margin:auto;
}

.header{
    text-align:center;
    margin-bottom:40px;
}

.header h1{
    color:#66c0f4;
    font-size:2.6rem;
    margin-bottom:10px;
}

.header p{
    color:#c7d5e0;
    font-size:1.05rem;
}

.store{
    background:#1b2838;
    border-radius:15px;
    padding:25px;
    margin-bottom:35px;
    box-shadow:0 8px 20px rgba(0,0,0,.35);
}

.storefront{
    color:#66c0f4;
    border-left:5px solid #66c0f4;
    padding-left:12px;
    margin-bottom:20px;
}

.game-card{
    display:flex;
    align-items:center;
    gap:20px;
    padding:18px;
    margin-bottom:18px;
    border-radius:12px;
    background:#22384f;
    transition:.25s;
}

.game-card:last-child{
    margin-bottom:0;
}

.game-card:hover{
    background:#2d4c69;
    transform:translateY(-2px);
}

.game-card img{
    width:220px;
    border-radius:8px;
    flex-shrink:0;
}

.game-info{
    flex:1;
}

.game-title{
    color:#ffffff;
    text-decoration:none;
    font-size:1.3rem;
    font-weight:700;
}

.game-title:hover{
    color:#66c0f4;
}

.price-row{
    display:flex;
    align-items:center;
    gap:15px;
    margin-top:15px;
    flex-wrap:wrap;
}

.old-price{
    color:#9aa3ad;
    text-decoration:line-through;
}

.new-price{
    color:#8ef58e;
    font-size:1.5rem;
    font-weight:bold;
}

.discount{
    background:#4CAF50;
    color:white;
    padding:6px 12px;
    border-radius:20px;
    font-weight:bold;
    font-size:.9rem;
}

.store-link{
    display:inline-block;
    margin-top:15px;
    color:#66c0f4;
    text-decoration:none;
    font-weight:600;
}

.store-link:hover{
    color:white;
}

@media(max-width:750px){

    .game-card{
        flex-direction:column;
        text-align:center;
    }

    .game-card img{
        width:100%;
        max-width:350px;
    }

    .price-row{
        justify-content:center;
    }
}
</style>
"#;

pub static HTML_BODY_HEADER: &str = r#"
<div class="header">
    <h1>🎮 Game Sale Alerts</h1>
    <p>
    One or more games have dropped below your target price.
    Games may appear multiple times if they're on sale across multiple storefronts.
    </p>
</div>
"#;