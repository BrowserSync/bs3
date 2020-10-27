$(function () {
    var conn = null;

    window.addEventListener("scroll", function() {
        if (conn) {
            conn.send(JSON.stringify({kind: "Scroll", x: 0, y: window.scrollY}));
        }
    });

    function log(msg) {
        var control = $('#log');
        control.html(control.html() + msg + '<br/>');
        control.scrollTop(control.scrollTop() + 1000);
    }

    function connect() {
        disconnect();
        var wsUri = (window.location.protocol == 'https:' && 'wss://' || 'ws://') + window.location.host + '/__bs3/ws/';
        conn = new WebSocket(wsUri);
        log('Connecting...');
        conn.onopen = function () {
            log('Connected.');
            update_ui();
        };
        conn.onmessage = function (e) {
            log('Received: ' + e.data);
            const parsed = JSON.parse(e.data);
            switch (parsed.kind) {
                case "Scroll": {
                    console.log("got scroll", parsed);
                    break;
                }
                case "FsNotify": {
                    console.log("fsnotify", parsed);
                    window.location.reload(true);
                    // setTimeout(() => {
                    // }, 500)
                    break;
                }
                default: {
                    console.log("unhandled %o", parsed);
                }
            }
        };
        conn.onclose = function () {
            log('Disconnected.');
            conn = null;
            update_ui();
        };
    }

    function disconnect() {
        if (conn != null) {
            log('Disconnecting...');
            conn.close();
            conn = null;
            update_ui();
        }
    }

    function update_ui() {
        var msg = '';
        if (conn == null) {
            $('#status').text('disconnected');
            $('#connect').html('Connect');
        } else {
            $('#status').text('connected (' + conn.protocol + ')');
            $('#connect').html('Disconnect');
        }
    }

    $('#connect').click(function () {
        if (conn == null) {
            connect();
        } else {
            disconnect();
        }
        update_ui();
        return false;
    });
    $('#send').click(function () {
        var text = $('#text').val();
        log('Sending: ' + text);
        conn.send(text);
        $('#text').val('').focus();
        return false;
    });
    $('#scroll').click(function () {
        log('Sending scroll message');
        conn.send(JSON.stringify({kind: "Scroll", x: 10, y: 20}));
        return false;
    });
    $('#text').keyup(function (e) {
        if (e.keyCode === 13) {
            $('#send').click();
            return false;
        }
    });
    connect();
})
