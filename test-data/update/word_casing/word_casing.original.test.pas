unit WordCasingFixture;

interface

implementation

procedure Run;
var
  HTTPCLIENT: string;
  HTTPClientHelper: string;
begin
  HTTPCLIENT := 'httpclient'; // httpclient in comment should remain
  HTTPClientHelper := HTTPCLIENT + IOS;
end;

end.
