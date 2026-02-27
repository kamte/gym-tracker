function confirmDelete(item) {
    return confirm('Are you sure you want to delete this ' + item + '? This cannot be undone.');
}

function addExerciseRow() {
    var template = document.getElementById('exercise-row-template');
    if (template) {
        var clone = template.content.cloneNode(true);
        document.getElementById('exercise-rows').appendChild(clone);
    }
}

// === Set Type Pills ===
function initSetTypePills() {
    document.querySelectorAll('.set-type-pills').forEach(function (container) {
        var radios = container.querySelectorAll('input[type="radio"]');
        radios.forEach(function (radio) {
            radio.addEventListener('change', function () {
                container.querySelectorAll('.pill').forEach(function (p) {
                    p.classList.remove('active');
                });
                if (radio.checked) {
                    radio.nextElementSibling.classList.add('active');
                }
            });
        });
    });
}

// === Rest Timer ===
var restTimer = {
    remaining: 0,
    total: 0,
    interval: null,
    paused: false,

    start: function (seconds) {
        this.remaining = seconds;
        this.total = seconds;
        this.paused = false;
        this.save();
        this.show();
        this.tick();
        var self = this;
        clearInterval(this.interval);
        this.interval = setInterval(function () { self.tick(); }, 1000);
    },

    tick: function () {
        if (this.paused) return;
        this.remaining--;
        this.save();
        this.updateDisplay();
        if (this.remaining <= 0) {
            this.done();
        }
    },

    done: function () {
        clearInterval(this.interval);
        this.interval = null;
        this.remaining = 0;
        this.save();
        this.updateDisplay();
        var bar = document.getElementById('rest-timer-bar');
        if (bar) bar.classList.add('timer-done');

        // Audio beep
        try {
            var ctx = new (window.AudioContext || window.webkitAudioContext)();
            for (var i = 0; i < 3; i++) {
                var osc = ctx.createOscillator();
                var gain = ctx.createGain();
                osc.connect(gain);
                gain.connect(ctx.destination);
                osc.frequency.value = 880;
                gain.gain.value = 0.3;
                osc.start(ctx.currentTime + i * 0.2);
                osc.stop(ctx.currentTime + i * 0.2 + 0.15);
            }
        } catch (e) {}

        // Vibration
        if (navigator.vibrate) {
            navigator.vibrate([200, 100, 200, 100, 200]);
        }

        // Auto-hide after 3 seconds
        var self = this;
        setTimeout(function () { self.skip(); }, 3000);
    },

    toggle: function () {
        this.paused = !this.paused;
        var btn = document.getElementById('rest-timer-toggle');
        if (btn) btn.textContent = this.paused ? 'Resume' : 'Pause';
    },

    adjust: function (delta) {
        this.remaining += delta;
        this.total += delta;
        if (this.remaining < 0) this.remaining = 0;
        if (this.total < 1) this.total = 1;
        this.save();
        this.updateDisplay();
    },

    skip: function () {
        clearInterval(this.interval);
        this.interval = null;
        this.remaining = 0;
        this.total = 0;
        localStorage.removeItem('restTimer');
        this.hide();
    },

    show: function () {
        var bar = document.getElementById('rest-timer-bar');
        if (bar) {
            bar.classList.remove('hidden', 'timer-done');
        }
    },

    hide: function () {
        var bar = document.getElementById('rest-timer-bar');
        if (bar) bar.classList.add('hidden');
    },

    updateDisplay: function () {
        var timeEl = document.getElementById('rest-timer-time');
        var progressEl = document.getElementById('rest-timer-progress');
        if (timeEl) {
            var mins = Math.floor(Math.max(0, this.remaining) / 60);
            var secs = Math.max(0, this.remaining) % 60;
            timeEl.textContent = mins + ':' + (secs < 10 ? '0' : '') + secs;
        }
        if (progressEl && this.total > 0) {
            var pct = Math.max(0, (this.remaining / this.total) * 100);
            progressEl.style.width = pct + '%';
        }
    },

    save: function () {
        localStorage.setItem('restTimer', JSON.stringify({
            remaining: this.remaining,
            total: this.total,
            timestamp: Date.now()
        }));
    },

    restore: function () {
        try {
            var data = JSON.parse(localStorage.getItem('restTimer'));
            if (!data || !data.remaining || data.remaining <= 0) return false;
            // Account for time elapsed while page was reloading
            var elapsed = Math.floor((Date.now() - data.timestamp) / 1000);
            var remaining = data.remaining - elapsed;
            if (remaining <= 0) {
                localStorage.removeItem('restTimer');
                return false;
            }
            this.remaining = remaining;
            this.total = data.total;
            this.paused = false;
            this.show();
            this.updateDisplay();
            var self = this;
            this.interval = setInterval(function () { self.tick(); }, 1000);
            return true;
        } catch (e) {
            return false;
        }
    }
};

// === Elapsed Workout Timer ===
function initElapsedTimer() {
    var el = document.getElementById('elapsed-timer');
    if (!el) return;
    var startedAt = el.getAttribute('data-started-at');
    if (!startedAt) return;

    // Parse "YYYY-MM-DD HH:MM:SS" as UTC
    var parts = startedAt.replace(' ', 'T') + 'Z';
    var startTime = new Date(parts).getTime();
    if (isNaN(startTime)) return;

    function update() {
        var now = Date.now();
        var diff = Math.floor((now - startTime) / 1000);
        if (diff < 0) diff = 0;
        var hours = Math.floor(diff / 3600);
        var mins = Math.floor((diff % 3600) / 60);
        var secs = diff % 60;
        var timeEl = document.getElementById('elapsed-time');
        if (timeEl) {
            if (hours > 0) {
                timeEl.textContent = hours + ':' + (mins < 10 ? '0' : '') + mins + ':' + (secs < 10 ? '0' : '') + secs;
            } else {
                timeEl.textContent = mins + ':' + (secs < 10 ? '0' : '') + secs;
            }
        }
    }

    update();
    setInterval(update, 1000);
}

// === Progress Chart (Canvas) ===
function initChart() {
    if (!window.progressData || !window.progressData.length) return;
    var canvas = document.getElementById('progress-chart');
    if (!canvas) return;

    window.currentMetric = 'max_weight';
    drawChart(canvas, window.progressData, 'max_weight');
}

function switchChart(metric, btn) {
    document.querySelectorAll('.chart-toggle').forEach(function (b) { b.classList.remove('active'); });
    btn.classList.add('active');
    window.currentMetric = metric;
    var canvas = document.getElementById('progress-chart');
    if (canvas) drawChart(canvas, window.progressData, metric);
}

function drawChart(canvas, data, metric) {
    var ctx = canvas.getContext('2d');
    var dpr = window.devicePixelRatio || 1;
    var rect = canvas.parentElement.getBoundingClientRect();
    var w = rect.width;
    var h = 250;

    canvas.width = w * dpr;
    canvas.height = h * dpr;
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';
    ctx.scale(dpr, dpr);

    ctx.clearRect(0, 0, w, h);

    if (data.length < 2) {
        ctx.fillStyle = '#6b7280';
        ctx.font = '14px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('Need at least 2 data points', w / 2, h / 2);
        return;
    }

    var values = data.map(function (d) { return d[metric]; });
    var minV = Math.min.apply(null, values);
    var maxV = Math.max.apply(null, values);
    if (minV === maxV) { minV -= 1; maxV += 1; }

    var pad = { top: 20, right: 20, bottom: 40, left: 60 };
    var plotW = w - pad.left - pad.right;
    var plotH = h - pad.top - pad.bottom;

    // Grid lines
    ctx.strokeStyle = '#e5e7eb';
    ctx.lineWidth = 1;
    var gridLines = 5;
    for (var i = 0; i <= gridLines; i++) {
        var y = pad.top + (plotH / gridLines) * i;
        ctx.beginPath();
        ctx.moveTo(pad.left, y);
        ctx.lineTo(w - pad.right, y);
        ctx.stroke();

        // Y labels
        var val = maxV - ((maxV - minV) / gridLines) * i;
        ctx.fillStyle = '#6b7280';
        ctx.font = '11px sans-serif';
        ctx.textAlign = 'right';
        ctx.fillText(Math.round(val), pad.left - 8, y + 4);
    }

    // Data points and line
    ctx.strokeStyle = '#4f46e5';
    ctx.lineWidth = 2;
    ctx.beginPath();

    var points = [];
    for (var j = 0; j < data.length; j++) {
        var x = pad.left + (j / (data.length - 1)) * plotW;
        var yVal = pad.top + plotH - ((values[j] - minV) / (maxV - minV)) * plotH;
        points.push({ x: x, y: yVal, date: data[j].date, value: values[j] });
        if (j === 0) ctx.moveTo(x, yVal);
        else ctx.lineTo(x, yVal);
    }
    ctx.stroke();

    // Fill area under line
    ctx.lineTo(points[points.length - 1].x, pad.top + plotH);
    ctx.lineTo(points[0].x, pad.top + plotH);
    ctx.closePath();
    ctx.fillStyle = 'rgba(79, 70, 229, 0.08)';
    ctx.fill();

    // Draw points
    ctx.fillStyle = '#4f46e5';
    for (var k = 0; k < points.length; k++) {
        ctx.beginPath();
        ctx.arc(points[k].x, points[k].y, 3, 0, Math.PI * 2);
        ctx.fill();
    }

    // X labels (show a few dates)
    ctx.fillStyle = '#6b7280';
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'center';
    var labelInterval = Math.max(1, Math.floor(data.length / 6));
    for (var m = 0; m < data.length; m += labelInterval) {
        var xLabel = pad.left + (m / (data.length - 1)) * plotW;
        ctx.fillText(data[m].date.substring(5), xLabel, h - pad.bottom + 18);
    }
    // Always show last label
    if ((data.length - 1) % labelInterval !== 0) {
        var xLast = pad.left + plotW;
        ctx.fillText(data[data.length - 1].date.substring(5), xLast, h - pad.bottom + 18);
    }

    // Hover tooltip
    canvas._points = points;
    canvas._metric = metric;
    canvas.onmousemove = function (e) {
        var rect = canvas.getBoundingClientRect();
        var mx = e.clientX - rect.left;
        var closest = null;
        var closestDist = Infinity;
        for (var i = 0; i < points.length; i++) {
            var dist = Math.abs(points[i].x - mx);
            if (dist < closestDist) {
                closestDist = dist;
                closest = points[i];
            }
        }
        if (closest && closestDist < 30) {
            canvas.style.cursor = 'crosshair';
            canvas.title = closest.date + ': ' + Math.round(closest.value * 10) / 10;
        } else {
            canvas.style.cursor = 'default';
            canvas.title = '';
        }
    };
}

// === Calendar tooltip hover ===
function initCalendarTooltips() {
    document.querySelectorAll('.calendar-day--has-workout').forEach(function (day) {
        day.addEventListener('mouseenter', function () {
            var tooltip = day.querySelector('.calendar-tooltip');
            if (tooltip) tooltip.style.display = 'block';
        });
        day.addEventListener('mouseleave', function () {
            var tooltip = day.querySelector('.calendar-tooltip');
            if (tooltip) tooltip.style.display = 'none';
        });
    });
}

// === Askama pluralize filter workaround ===
// (handled in template with custom logic)

// === Init ===
document.addEventListener('DOMContentLoaded', function () {
    // Auto-hide alerts after 5 seconds
    document.querySelectorAll('.alert-success').forEach(function (alert) {
        setTimeout(function () {
            alert.style.transition = 'opacity 0.5s';
            alert.style.opacity = '0';
            setTimeout(function () { alert.remove(); }, 500);
        }, 5000);
    });

    // Auto-hide PR banner after 5 seconds
    var prBanner = document.getElementById('pr-banner');
    if (prBanner) {
        setTimeout(function () {
            prBanner.style.transition = 'opacity 0.5s';
            prBanner.style.opacity = '0';
            setTimeout(function () { prBanner.remove(); }, 500);
        }, 5000);
    }

    initSetTypePills();
    initElapsedTimer();
    initChart();
    initCalendarTooltips();

    // Rest timer: restore saved state or auto-start from query param
    if (window.workoutData) {
        var restored = restTimer.restore();
        if (!restored && window.workoutData.justLogged && window.workoutData.restSeconds > 0) {
            restTimer.start(window.workoutData.restSeconds);
        }
    }
});

// Redraw chart on window resize
window.addEventListener('resize', function () {
    if (window.progressData && window.progressData.length >= 2) {
        var canvas = document.getElementById('progress-chart');
        if (canvas) drawChart(canvas, window.progressData, window.currentMetric || 'max_weight');
    }
});
