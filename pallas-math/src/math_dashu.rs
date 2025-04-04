/*!
# Cardano Math functions using the num-bigint crate
 */

use crate::math::{Error, ExpCmpOrdering, ExpOrdering, FixedPrecision, DEFAULT_PRECISION};
use dashu_base::{Abs, DivRem, Sign};
use dashu_int::{IBig, UBig};
use regex::Regex;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::str::FromStr;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct Decimal {
    precision: u64,
    precision_multiplier: IBig,
    data: IBig,
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.precision == other.precision
            && self.precision_multiplier == other.precision_multiplier
            && self.data == other.data
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.precision != other.precision
            || self.precision_multiplier != other.precision_multiplier
        {
            return None;
        }
        Some(self.data.cmp(&other.data))
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            print_fixedp(
                &self.data,
                &self.precision_multiplier,
                self.precision as usize,
            )
        )
    }
}

impl From<u64> for Decimal {
    fn from(n: u64) -> Self {
        let mut result = Decimal::new(DEFAULT_PRECISION);
        result.data = IBig::from(n) * &result.precision_multiplier;
        result
    }
}

impl From<i64> for Decimal {
    fn from(n: i64) -> Self {
        let mut result = Decimal::new(DEFAULT_PRECISION);
        result.data = IBig::from(n) * &result.precision_multiplier;
        result
    }
}

impl From<IBig> for Decimal {
    fn from(n: IBig) -> Self {
        let mut result = Decimal::new(DEFAULT_PRECISION);
        result.data = n * &result.precision_multiplier;
        result
    }
}

impl From<&IBig> for Decimal {
    fn from(n: &IBig) -> Self {
        let mut result = Decimal::new(DEFAULT_PRECISION);
        result.data = n * &result.precision_multiplier;
        result
    }
}

impl From<UBig> for Decimal {
    fn from(n: UBig) -> Self {
        let mut result = Decimal::new(DEFAULT_PRECISION);
        result.data = IBig::from(n) * &result.precision_multiplier;
        result
    }
}

impl From<&UBig> for Decimal {
    fn from(n: &UBig) -> Self {
        let mut result = Decimal::new(DEFAULT_PRECISION);
        result.data = IBig::from(n.clone()) * &result.precision_multiplier;
        result
    }
}

impl From<&[u8]> for Decimal {
    fn from(n: &[u8]) -> Self {
        Decimal::from(UBig::from_be_bytes(n))
    }
}

impl Neg for Decimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = -self.data;
        result
    }
}

// Implement Neg for a reference to Decimal
impl Neg for &Decimal {
    type Output = Decimal;

    fn neg(self) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = -&self.data;
        result
    }
}

impl Abs for Decimal {
    type Output = Self;

    fn abs(self) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = self.data.abs();
        result
    }
}

// Implement Abs for a reference to Decimal
impl Abs for &Decimal {
    type Output = Decimal;

    fn abs(self) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = (&self.data).abs();
        result
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = &self.data * &rhs.data;
        scale(&mut result.data);
        result
    }
}

impl MulAssign for Decimal {
    fn mul_assign(&mut self, rhs: Self) {
        self.data *= &rhs.data;
        scale(&mut self.data);
    }
}

// Implement Mul for a reference to Decimal
impl<'b> Mul<&'b Decimal> for &Decimal {
    type Output = Decimal;

    fn mul(self, rhs: &'b Decimal) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = &self.data * &rhs.data;
        scale(&mut result.data);
        result
    }
}

impl<'b> MulAssign<&'b Decimal> for &mut Decimal {
    fn mul_assign(&mut self, rhs: &'b Decimal) {
        self.data *= &rhs.data;
        scale(&mut self.data);
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        div(&mut result.data, &self.data, &rhs.data);
        result
    }
}

impl DivAssign for Decimal {
    fn div_assign(&mut self, rhs: Self) {
        let temp = self.data.clone();
        div(&mut self.data, &temp, &rhs.data);
    }
}

// Implement Div for a reference to Decimal
impl<'b> Div<&'b Decimal> for &Decimal {
    type Output = Decimal;

    fn div(self, rhs: &'b Decimal) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        div(&mut result.data, &self.data, &rhs.data);
        result
    }
}

impl<'b> DivAssign<&'b Decimal> for &mut Decimal {
    fn div_assign(&mut self, rhs: &'b Decimal) {
        let temp = self.data.clone();
        div(&mut self.data, &temp, &rhs.data);
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = &self.data - &rhs.data;
        result
    }
}

impl SubAssign for Decimal {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= &rhs.data;
    }
}

// Implement Sub for a reference to Decimal
impl<'b> Sub<&'b Decimal> for &Decimal {
    type Output = Decimal;

    fn sub(self, rhs: &'b Decimal) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = &self.data - &rhs.data;
        result
    }
}

impl<'b> SubAssign<&'b Decimal> for &mut Decimal {
    fn sub_assign(&mut self, rhs: &'b Decimal) {
        self.data -= &rhs.data;
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = &self.data + &rhs.data;
        result
    }
}

impl AddAssign for Decimal {
    fn add_assign(&mut self, rhs: Self) {
        self.data += &rhs.data;
    }
}

// Implement Add for a reference to Decimal
impl<'b> Add<&'b Decimal> for &Decimal {
    type Output = Decimal;

    fn add(self, rhs: &'b Decimal) -> Self::Output {
        let mut result = Decimal::new(self.precision);
        result.data = &self.data + &rhs.data;
        result
    }
}

impl<'b> AddAssign<&'b Decimal> for &mut Decimal {
    fn add_assign(&mut self, rhs: &'b Decimal) {
        self.data += &rhs.data;
    }
}

impl FixedPrecision for Decimal {
    fn new(precision: u64) -> Self {
        let mut precision_multiplier = IBig::from(10);
        precision_multiplier = precision_multiplier.pow(precision as usize);
        let data = IBig::from(0);
        Decimal {
            precision,
            precision_multiplier,
            data,
        }
    }

    fn from_str(s: &str, precision: u64) -> Result<Self, Error> {
        // assert that s contains only digits using a regex
        if !DIGITS_REGEX.is_match(s) {
            return Err(Error::RegexFailure(regex::Error::Syntax(
                "string contained non-digits".to_string(),
            )));
        }

        let mut decimal = Decimal::new(precision);
        decimal.data = IBig::from_str(s).unwrap();
        Ok(decimal)
    }

    fn precision(&self) -> u64 {
        self.precision
    }

    fn exp(&self) -> Self {
        let mut exp_x = Decimal::new(self.precision);
        ref_exp(&mut exp_x.data, &self.data);
        exp_x
    }

    fn ln(&self) -> Self {
        let mut ln_x = Decimal::new(self.precision);
        if ref_ln(&mut ln_x.data, &self.data) {
            ln_x
        } else {
            panic!("ln of a value in (-inf,0] is undefined")
        }
    }

    /// Compute the power of a Decimal approximation using x^y = exp(y * ln x) formula
    /// While not exact, this is a more performant way to compute the power of a Decimal
    fn pow(&self, rhs: &Self) -> Self {
        let mut pow_x = Decimal::new(self.precision);
        ref_pow(&mut pow_x.data, &self.data, &rhs.data);
        pow_x
    }

    fn exp_cmp(&self, max_n: u64, bound_self: i64, compare: &Self) -> ExpCmpOrdering {
        let mut output = Decimal::new(self.precision);
        ref_exp_cmp(
            &mut output.data,
            max_n,
            &self.data,
            bound_self,
            &compare.data,
        )
    }

    fn round(&self) -> Self {
        let mut result = self.clone();
        let half = &self.precision_multiplier / IBig::from(2);
        let remainder = &self.data % &self.precision_multiplier;
        if (&remainder).abs() >= half {
            if self.data.sign() == Sign::Negative {
                result.data -= &self.precision_multiplier + remainder;
            } else {
                result.data += &self.precision_multiplier - remainder;
            }
        } else {
            result.data -= remainder;
        }
        result
    }

    fn floor(&self) -> Self {
        let mut result = self.clone();
        let remainder = &self.data % &self.precision_multiplier;
        if self.data.sign() == Sign::Negative && remainder != ZERO.value {
            result.data -= &self.precision_multiplier;
        }
        result.data -= remainder;
        result
    }

    fn ceil(&self) -> Self {
        let mut result = self.clone();
        let remainder = &self.data % &self.precision_multiplier;
        if self.data.sign() == Sign::Positive && remainder != ZERO.value {
            result.data += &self.precision_multiplier;
        }
        result.data -= remainder;
        result
    }

    fn trunc(&self) -> Self {
        let mut result = self.clone();
        result.data -= &self.data % &self.precision_multiplier;
        result
    }
}

fn print_fixedp(n: &IBig, precision: &IBig, width: usize) -> String {
    let (mut temp_q, mut temp_r) = n.div_rem(precision);

    let is_negative_q = temp_q < ZERO.value;
    let is_negative_r = temp_r < ZERO.value;

    if is_negative_q {
        temp_q = temp_q.abs();
    }
    if is_negative_r {
        temp_r = temp_r.abs();
    }

    let mut s = String::new();
    if is_negative_q || is_negative_r {
        s.push('-');
    }
    s.push_str(&temp_q.to_string());
    s.push('.');
    let r = temp_r.to_string();
    let r_len = r.len();
    // fill with zeroes up to width for the fractional part
    if r_len < width {
        s.push_str(&"0".repeat(width - r_len));
    }
    s.push_str(&r);
    s
}

struct Constant {
    value: IBig,
}

impl Constant {
    pub fn new(init: fn() -> IBig) -> Constant {
        Constant { value: init() }
    }
}

unsafe impl Sync for Constant {}
unsafe impl Send for Constant {}

static DIGITS_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?\d+$").unwrap());
static TEN: LazyLock<Constant> = LazyLock::new(|| Constant::new(|| IBig::from(10)));
static PRECISION: LazyLock<Constant> =
    LazyLock::new(|| Constant::new(|| TEN.value.clone().pow(34)));
static EPS: LazyLock<Constant> = LazyLock::new(|| Constant::new(|| TEN.value.clone().pow(34 - 24)));
static ONE: LazyLock<Constant> =
    LazyLock::new(|| Constant::new(|| IBig::from(1) * &PRECISION.value));
static ZERO: LazyLock<Constant> = LazyLock::new(|| Constant::new(|| IBig::from(0)));
static E: LazyLock<Constant> = LazyLock::new(|| {
    Constant::new(|| {
        let mut e = IBig::from(0);
        ref_exp(&mut e, &ONE.value);
        e
    })
});

fn div_round_ceil(x: &IBig, y: &IBig) -> IBig {
    let (q, r) = x.div_rem(y);
    if q.sign() == Sign::Positive && r != IBig::ZERO {
        q + IBig::ONE
    } else {
        q
    }
}

/// Entry point for 'exp' approximation. First does the scaling of 'x' to [0,1]
/// and then calls the continued fraction approximation function.
fn ref_exp(rop: &mut IBig, x: &IBig) -> i32 {
    let mut iterations = 0;
    match x.cmp(&ZERO.value) {
        Ordering::Equal => {
            // rop = 1
            rop.clone_from(&ONE.value);
        }
        Ordering::Less => {
            let x_ = -x;
            let mut temp = IBig::from(0);
            iterations = ref_exp(&mut temp, &x_);
            // rop = 1 / temp
            div(rop, &ONE.value, &temp);
        }
        Ordering::Greater => {
            let n_exponent = div_round_ceil(x, &PRECISION.value);
            let x_ = x / &n_exponent;
            iterations = mp_exp_taylor(rop, 1000, &x_, &EPS.value);

            // rop = rop.pow(n)
            let n_exponent_i64: i64 = i64::try_from(&n_exponent).expect("n_exponent to_i64 failed");
            ipow(rop, &rop.clone(), n_exponent_i64);
        }
    }

    iterations
}

/// Division with quotent and remainder
#[inline]
fn div_qr(q: &mut IBig, r: &mut IBig, x: &IBig, y: &IBig) {
    (*q, *r) = x.div_rem(y);
}

/// Division
pub fn div(rop: &mut IBig, x: &IBig, y: &IBig) {
    let mut temp_q = IBig::from(0);
    let mut temp_r = IBig::from(0);
    let mut temp: IBig;
    div_qr(&mut temp_q, &mut temp_r, x, y);

    temp = &temp_q * &PRECISION.value;
    temp_r = &temp_r * &PRECISION.value;
    let temp_r2 = temp_r.clone();
    div_qr(&mut temp_q, &mut temp_r, &temp_r2, y);

    temp += &temp_q;
    *rop = temp;
}

/// Taylor / MacLaurin series approximation
fn mp_exp_taylor(rop: &mut IBig, max_n: i32, x: &IBig, epsilon: &IBig) -> i32 {
    let mut divisor = ONE.value.clone();
    let mut last_x = ONE.value.clone();
    rop.clone_from(&ONE.value);
    let mut n = 0;
    while n < max_n {
        let mut next_x = x * &last_x;
        scale(&mut next_x);
        let next_x2 = next_x.clone();
        div(&mut next_x, &next_x2, &divisor);

        if (&next_x).abs() < epsilon.abs() {
            break;
        }

        divisor += &ONE.value;
        *rop = &*rop + &next_x;
        last_x.clone_from(&next_x);
        n += 1;
    }

    n
}

pub(crate) fn scale(rop: &mut IBig) {
    let mut temp = IBig::from(0);
    let mut a = IBig::from(0);
    div_qr(&mut a, &mut temp, rop, &PRECISION.value);
    if *rop < ZERO.value && temp != ZERO.value {
        a -= IBig::ONE;
    }
    *rop = a;
}

/// Integer power internal function
fn ipow_(rop: &mut IBig, x: &IBig, n: i64) {
    if n == 0 {
        rop.clone_from(&ONE.value);
    } else if n % 2 == 0 {
        let mut res = IBig::from(0);
        ipow_(&mut res, x, n / 2);
        *rop = &res * &res;
        scale(rop);
    } else {
        let mut res = IBig::from(0);
        ipow_(&mut res, x, n - 1);
        *rop = res * x;
        scale(rop);
    }
}

/// Integer power
fn ipow(rop: &mut IBig, x: &IBig, n: i64) {
    if n < 0 {
        let mut temp = IBig::from(0);
        ipow_(&mut temp, x, -n);
        div(rop, &ONE.value, &temp);
    } else {
        ipow_(rop, x, n);
    }
}

/// Compute an approximation of 'ln(1 + x)' via continued fractions. Either for a
///    maximum of 'maxN' iterations or until the absolute difference between two
///    succeeding convergents is smaller than 'eps'. Assumes 'x' to be within
///    [1,e).
fn mp_ln_n(rop: &mut IBig, max_n: i32, x: &IBig, epsilon: &IBig) {
    let mut ba: IBig;
    let mut aa: IBig;
    let mut ab: IBig;
    let mut bb: IBig;
    let mut a_: IBig;
    let mut b_: IBig;
    let mut diff: IBig;
    let mut convergent: IBig = IBig::from(0);
    let mut last: IBig = IBig::from(0);
    let mut first = true;
    let mut n = 1;

    let mut a: IBig;
    let mut b = ONE.value.clone();

    let mut an_m2 = ONE.value.clone();
    let mut bn_m2 = IBig::from(0);
    let mut an_m1 = IBig::from(0);
    let mut bn_m1 = ONE.value.clone();

    let mut curr_a = 1;

    while n <= max_n + 2 {
        let curr_a_2 = curr_a * curr_a;
        a = x * IBig::from(curr_a_2);
        if n > 1 && n % 2 == 1 {
            curr_a += 1;
        }

        ba = &b * &an_m1;
        scale(&mut ba);
        aa = &a * &an_m2;
        scale(&mut aa);
        a_ = &ba + &aa;

        bb = &b * &bn_m1;
        scale(&mut bb);
        ab = &a * &bn_m2;
        scale(&mut ab);
        b_ = &bb + &ab;

        div(&mut convergent, &a_, &b_);

        if first {
            first = false;
        } else {
            diff = &convergent - &last;
            if diff.abs() < epsilon.abs() {
                break;
            }
        }

        last.clone_from(&convergent);

        n += 1;
        an_m2.clone_from(&an_m1);
        bn_m2.clone_from(&bn_m1);
        an_m1.clone_from(&a_);
        bn_m1.clone_from(&b_);

        b += &ONE.value;
    }

    *rop = convergent;
}

fn find_e(x: &IBig) -> i64 {
    let mut x_: IBig = IBig::from(0);
    let mut x__: IBig = E.value.clone();

    div(&mut x_, &ONE.value, &E.value);

    let mut l = -1;
    let mut u = 1;
    while &x_ > x || &x__ < x {
        x_ = &x_ * &x_;
        scale(&mut x_);

        x__ = &x__ * &x__;
        scale(&mut x__);

        l *= 2;
        u *= 2;
    }

    while l + 1 != u {
        let mid = l + ((u - l) / 2);

        ipow(&mut x_, &E.value, mid);
        if x < &x_ {
            u = mid;
        } else {
            l = mid;
        }
    }
    l
}

/// Entry point for 'ln' approximation. First does the necessary scaling, and
/// then calls the continued fraction calculation. For any value outside the
/// domain, i.e., 'x in (-inf,0]', the function returns '-INFINITY'.
fn ref_ln(rop: &mut IBig, x: &IBig) -> bool {
    let mut factor = IBig::from(0);
    let mut x_ = IBig::from(0);
    if x <= &ZERO.value {
        return false;
    }

    let n = find_e(x);

    *rop = IBig::from(n);
    *rop = &*rop * &PRECISION.value;
    ref_exp(&mut factor, rop);

    div(&mut x_, x, &factor);

    x_ = &x_ - &ONE.value;

    let x_2 = x_.clone();
    mp_ln_n(&mut x_, 1000, &x_2, &EPS.value);
    *rop = &*rop + &x_;

    true
}

fn ref_pow(rop: &mut IBig, base: &IBig, exponent: &IBig) {
    /* x^y = exp(y * ln x) */
    let mut tmp: IBig = IBig::from(0);

    if exponent == &ZERO.value || base == &ONE.value {
        // any base to the power of zero is one, or 1 to any power is 1
        *rop = ONE.value.clone();
        return;
    }
    if exponent == &ONE.value {
        // any base to the power of one is the base
        *rop = base.clone();
        return;
    }
    if base == &ZERO.value && exponent > &ZERO.value {
        // zero to any positive power is zero
        *rop = &ZERO.value * &PRECISION.value;
        return;
    }
    if base == &ZERO.value && exponent < &ZERO.value {
        panic!("zero to a negative power is undefined");
    }
    if base < &ZERO.value {
        // negate the base and calculate the power
        let neg_base = base.neg();
        let ref_ln = ref_ln(&mut tmp, &neg_base);
        debug_assert!(ref_ln);
        tmp *= exponent;
        scale(&mut tmp);
        let mut tmp_rop = IBig::from(0);
        ref_exp(&mut tmp_rop, &tmp);
        let (_, rem) = (exponent / &PRECISION.value).div_rem(&IBig::from(2));
        // check if rem is even
        *rop = if rem == IBig::ZERO { tmp_rop } else { -tmp_rop };
    } else {
        // base is positive, ref_ln result is valid
        let ref_ln = ref_ln(&mut tmp, base);
        debug_assert!(ref_ln);
        tmp *= exponent;
        scale(&mut tmp);
        ref_exp(rop, &tmp);
    }
}

/// `bound_x` is the bound for exp in the interval x is chosen from
/// `compare` the value to compare to
///
/// if the result is GT, then the computed value is guaranteed to be greater, if
/// the result is LT, the computed value is guaranteed to be less than
/// `compare`. In the case of `UNKNOWN` no conclusion was possible for the
/// selected precision.
///
/// Lagrange remainder require knowledge of the maximum value to compute the
/// maximal error of the remainder.
fn ref_exp_cmp(
    rop: &mut IBig,
    max_n: u64,
    x: &IBig,
    bound_x: i64,
    compare: &IBig,
) -> ExpCmpOrdering {
    rop.clone_from(&ONE.value);
    let mut n = 0u64;
    let mut divisor: IBig;
    let mut next_x: IBig;
    let mut error: IBig;
    let mut upper: IBig;
    let mut lower: IBig;
    let mut error_term: IBig;

    divisor = ONE.value.clone();
    error = x.clone();

    let mut estimate = ExpOrdering::UNKNOWN;
    while n < max_n {
        next_x = error.clone();
        if (&next_x).abs() < (&EPS.value).abs() {
            break;
        }
        divisor += &ONE.value;

        // update error estimation, this is initially bound_x * x and in general
        // bound_x * x^(n+1)/(n + 1)!  we use `error` to store the x^n part and a
        // single integral multiplication with the bound
        error *= x;
        scale(&mut error);
        let e2 = error.clone();
        div(&mut error, &e2, &divisor);
        error_term = &error * IBig::from(bound_x);
        *rop = &*rop + &next_x;

        /* compare is guaranteed to be above overall result */
        upper = &*rop + &error_term;
        if compare > &upper {
            estimate = ExpOrdering::GT;
            n += 1;
            break;
        }

        /* compare is guaranteed to be below overall result */
        lower = &*rop - &error_term;
        if compare < &lower {
            estimate = ExpOrdering::LT;
            n += 1;
            break;
        }
        n += 1;
    }

    let mut approx = Decimal::new(DEFAULT_PRECISION);
    approx.data = rop.clone();

    ExpCmpOrdering {
        iterations: n,
        estimation: estimate,
        approx,
    }
}
