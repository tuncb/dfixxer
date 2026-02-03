unit UnaryMinusDemo;
interface
const
  A = -1;
  B = -1.23;
  C = -$FF;
  D = -1E-3;
var
  I: Integer;
  R: Double;
implementation
begin
  I := -1;
  I := a * -2;
  I := -a + 1;
  I := -(a + 1);
  I := -Foo(1);
  I := -Foo;
  I := a--b;
  R := -1.0E+2;
end.
